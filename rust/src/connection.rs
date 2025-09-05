use super::{Error, Stats};
use crate::stats::ConnectionState;
use serde::Deserialize;
use serde_json::Value;
use std::io::ErrorKind;
use std::time::{Duration, SystemTime};
use std::{
    collections::HashMap,
    net::TcpStream,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};
use strum_macros::{AsRefStr, Display};
use tungstenite::{Message, WebSocket, stream::MaybeTlsStream};

#[derive(AsRefStr, Clone, Debug, Display)]
pub enum Format {
    #[strum(serialize = "json")]
    Json,
}

#[derive(Clone, Debug)]
struct Response {
    error: Option<String>,
}

pub struct Connection {
    cache: Arc<Mutex<Cache>>,
    next_transaction_id: AtomicU64,
    requests: Arc<Mutex<HashMap<u64, Option<Response>>>>,
    socket: Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>,
}

#[derive(Debug, Default, Deserialize)]
struct Cache {
    version: String,
    stats: Stats,
    keys: HashMap<String, Value>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct SparseStats {
    started: Option<String>,
    reads: Option<u32>,
    writes: Option<u32>,
    updates: Option<u32>,
    errors: Option<u32>,
    connection: Option<ConnectionState>,
}

#[derive(Debug, Default, Deserialize)]
struct SparseCache {
    stats: Option<SparseStats>,
    version: Option<String>,
    keys: Option<HashMap<String, Value>>,
}

impl Connection {
    pub(crate) fn new(socket: Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>) -> Self {
        let next_transaction_id = AtomicU64::new(1);
        let cache = Arc::new(Mutex::new(Cache::default()));
        let requests = Arc::new(Mutex::new(HashMap::new()));

        // Spawn a thread to handle incoming messages
        let socket_2 = socket.clone();
        let cache_2 = cache.clone();
        let requests_2 = requests.clone();
        std::thread::spawn(move || {
            loop {
                let maybe_message = {
                    if let Ok(mut socket) = socket_2.lock() {
                        match socket.read() {
                            Ok(message) => Some(message),
                            Err(e) => match e {
                                tungstenite::Error::Io(ref e) if e.kind() == ErrorKind::WouldBlock => None,
                                _ => panic!("{e:?}"),
                            },
                        }
                    } else {
                        continue;
                    }
                };
                let message = match maybe_message {
                    Some(message) => message,
                    None => {
                        std::thread::sleep(Duration::from_millis(20));
                        continue;
                    }
                };
                if let Message::Text(ref message) = message {
                    let incoming: HashMap<String, Value> = serde_json::from_slice(message.as_bytes()).unwrap();
                    eprintln!("** got {incoming:?}");

                    if let Some(Value::Number(transaction)) = incoming.get("transaction") {
                        let transaction: u64 = transaction.as_u64().unwrap();
                        if let Some(response) = incoming.get("response") {
                            let mut requests = requests_2.lock().unwrap();
                            if response.is_string() && response.to_string().is_empty() {
                                requests.insert(transaction, Some(Response{error: None}));
                            } else if let Value::Object(response) = response {
                                if let Some(error_msg) = response.get("error") {
                                    requests.insert(transaction, Some(Response { error: Some(error_msg.to_string()) }));
                                }
                            }
                        }
                        continue;
                    }

                    let payload: SparseCache = serde_json::from_slice(message.as_bytes()).unwrap();
                    if let Ok(mut cache) = cache_2.lock() {
                        Self::merge(&mut cache, &payload);
                    }
                }
            }
        });

        Self {
            cache,
            next_transaction_id,
            requests,
            socket,
        }
    }

    pub fn add_node(&mut self, id: &str, name: &str, upstream: bool, token: Option<String>) -> Result<(), Error> {
        // Send request
        let upstream_arg = if upstream { "yes".to_string() } else { "no".to_string() };
        let token_arg = token.unwrap_or("$uuid".to_string());
        let args = vec![id.to_string(), name.to_string(), upstream_arg, token_arg];
        let transaction = self.send(Format::Json, "nodes", &args)?;

        // Wait for response
        let _response = self.wait_for_response(transaction, Duration::from_secs(5))?;
        Ok(())
    }

    pub fn get(&self, key: &str, default_value: Option<Value>) -> Result<Option<Value>, Error> {
        let cache = self.cache.lock()?;
        let value = match cache.keys.get(key) {
            Some(value) => Some(value.clone()),
            None => default_value,
        };
        Ok(value)
    }

    pub fn keys(&self) -> Result<Vec<String>, Error> {
        let mut vec = vec![];
        let cache = self.cache.lock()?;
        for (key, _) in cache.keys.iter() {
            vec.push(key.clone());
        }
        Ok(vec)
    }

    fn merge(target: &mut Cache, source: &SparseCache) {
        if let Some(ref stats) = source.stats {
            if let Some(ref started) = stats.started {
                target.stats.started = started.clone();
            }
            if let Some(reads) = stats.reads {
                target.stats.reads = reads;
            }
            if let Some(writes) = stats.writes {
                target.stats.writes = writes;
            }
            if let Some(updates) = stats.updates {
                target.stats.updates = updates;
            }
            if let Some(errors) = stats.errors {
                target.stats.errors = errors;
            }
            if let Some(ref connection) = stats.connection {
                target.stats.connection = connection.clone();
            }
        }
        if let Some(ref version) = source.version {
            target.version = version.clone();
        }
        if let Some(ref keys) = source.keys {
            for (k, v) in keys.iter() {
                target.keys.insert(k.to_string(), v.clone());
            }
        }
    }

    pub fn put(&mut self, key: &str, value: &str) -> Result<(), Error> {
        let args = vec![format!("\"{key}\""), value.to_string()];
        let _ = self.send(Format::Json, "put", &args)?;
        Ok(())
    }

    pub fn send(&mut self, response_format: Format, cmd: &str, args: &[String]) -> Result<u64, Error> {
        let mut cmd = cmd.to_string();
        for arg in args {
            cmd = format!("{cmd} \"{arg}\"");
        }

        let mut msg = HashMap::new();
        let transaction_id = self.next_transaction_id.fetch_add(1, Ordering::SeqCst);
        msg.insert("format".to_string(), Value::String(response_format.to_string()));
        msg.insert("transaction".to_string(), Value::from(transaction_id));
        msg.insert("command".to_string(), Value::String(cmd));

        {
            let mut requests = self.requests.lock()?;
            requests.insert(transaction_id, None);
        }

        let msg_as_json = serde_json::to_string(&msg)?;
        eprintln!("** sending {msg_as_json}");
        let message = Message::text(msg_as_json);
        self.send_message(message)?;
        Ok(transaction_id)
    }

    fn send_message(&mut self, message: Message) -> Result<(), Error> {
        let mut socket = self.socket.lock()?;
        socket.send(message)?;
        Ok(())
    }

    pub fn stats(&self) -> Result<Stats, Error> {
        let cache = self.cache.lock()?;
        Ok(cache.stats.clone())
    }

    pub fn version(&self) -> Result<String, Error> {
        let cache = self.cache.lock()?;
        Ok(cache.version.clone())
    }

    pub fn wait_for_open(&self, timeout: Duration) -> Result<(), Error> {
        let start_time = SystemTime::now();
        let sleep_for = Duration::from_millis(100);
        while SystemTime::now().duration_since(start_time)? < timeout {
            {
                let cache = self.cache.lock()?;
                if !cache.version.is_empty() {
                    return Ok(());
                }
            }
            std::thread::sleep(sleep_for);
        }
        Err(Error::Timeout("Timed out waiting for open".to_string()))
    }

    fn wait_for_response(&self, transaction: u64, timeout: Duration) -> Result<Response, Error> {
        let start_time = SystemTime::now();
        let sleep_for = Duration::from_millis(100);
        while SystemTime::now().duration_since(start_time)? < timeout {
            {
                let requests = self.requests.lock()?;
                if let Some(response) = requests.get(&transaction) {
                    match response {
                        None => continue,
                        Some(response) => {
                            match response.error.as_ref() {
                                None => return Ok(response.clone()),
                                Some(error_msg) => return Err(Error::Default(error_msg.clone())),
                            }
                        }
                    }
                }
            }
            std::thread::sleep(sleep_for);
        }
        Err(Error::Timeout("Timed out waiting for response".to_string()))
    }
}
