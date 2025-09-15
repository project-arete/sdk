use crate::{Cache, Error, Stats, System, stats::ConnectionState, system};
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    io::ErrorKind,
    net::TcpStream,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    time::{Duration, SystemTime},
};
use strum_macros::{AsRefStr, Display};
use tungstenite::{Message, WebSocket, stream::MaybeTlsStream};

pub const DEFAULT_TIMEOUT_SECS: u64 = 5;

#[derive(AsRefStr, Clone, Debug, Display)]
pub enum Format {
    #[strum(serialize = "json")]
    Json,
}

#[derive(Clone, Debug)]
pub(crate) struct Response {
    error: Option<String>,
}

struct State {
    cache: Arc<Mutex<Cache>>,
    next_transaction_id: AtomicU64,
    requests: Arc<Mutex<HashMap<u64, Option<Response>>>>,
    socket: Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>,
    subscribers: Arc<Mutex<Vec<Sender<Cache>>>>,
}

#[derive(Clone)]
pub struct Client {
    state: Arc<State>,
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

impl From<SparseCache> for Cache {
    fn from(other: SparseCache) -> Self {
        let mut cache = Cache::default();
        if let Some(stats) = other.stats {
            if let Some(started) = stats.started {
                cache.stats.started = started.clone();
            }
            if let Some(reads) = stats.reads {
                cache.stats.reads = reads;
            }
            if let Some(writes) = stats.writes {
                cache.stats.writes = writes;
            }
            if let Some(updates) = stats.updates {
                cache.stats.updates = updates;
            }
            if let Some(errors) = stats.errors {
                cache.stats.errors = errors;
            }
            if let Some(connection) = stats.connection {
                cache.stats.connection = connection.clone();
            }
        }
        if let Some(version) = other.version {
            cache.version = version.clone();
        }
        if let Some(keys) = other.keys {
            cache.keys = keys.clone();
        }
        cache
    }
}

impl Client {
    pub(crate) fn new(socket: Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>) -> Self {
        let next_transaction_id = AtomicU64::new(1);
        let cache = Arc::new(Mutex::new(Cache::default()));
        let requests = Arc::new(Mutex::new(HashMap::new()));
        let subscribers = Arc::new(Mutex::<Vec<Sender<Cache>>>::new(vec![]));

        // Spawn a thread to handle incoming messages
        let socket_2 = socket.clone();
        let cache_2 = cache.clone();
        let requests_2 = requests.clone();
        let subscribers_2 = subscribers.clone();
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

                    if let Some(Value::Number(transaction)) = incoming.get("transaction") {
                        let transaction: u64 = transaction.as_u64().unwrap();
                        if let Some(response) = incoming.get("response") {
                            let mut requests = requests_2.lock().unwrap();
                            if response.is_string() && response.as_str().unwrap_or_default().is_empty() {
                                requests.insert(transaction, Some(Response { error: None }));
                            } else if let Value::Object(response) = response {
                                if let Some(error_msg) = response.get("error") {
                                    requests.insert(
                                        transaction,
                                        Some(Response {
                                            error: Some(error_msg.to_string()),
                                        }),
                                    );
                                }
                            }
                        }
                        continue;
                    }

                    // Cache the new update
                    let payload: SparseCache = serde_json::from_slice(message.as_bytes()).unwrap();
                    if let Ok(mut cache) = cache_2.lock() {
                        Self::merge(&mut cache, &payload);
                    }

                    // Notify subscribers of the new update
                    {
                        let subscribers = subscribers_2.lock().unwrap();
                        let event: Cache = payload.into();
                        for tx in subscribers.iter() {
                            tx.send(event.clone()).unwrap();
                        }
                    }
                }
            }
        });

        let state = Arc::new(State {
            cache,
            next_transaction_id,
            requests,
            socket,
            subscribers,
        });
        Self { state }
    }

    pub fn add_consumer(&mut self, node_id: &str, context_id: &str, profile: &str) -> Result<(), Error> {
        let system_id = system::get_system_id()?;
        let args = vec![
            system_id.to_string(),
            node_id.to_string(),
            context_id.to_string(),
            profile.to_string(),
        ];
        let transaction = self.send(Format::Json, "consumers", &args)?;
        let _response = self.wait_for_response(transaction, Duration::from_secs(DEFAULT_TIMEOUT_SECS))?;
        Ok(())
    }

    pub fn get(&self, key: &str, default_value: Option<Value>) -> Result<Option<Value>, Error> {
        let cache = self.state.cache.lock()?;
        let value = match cache.keys.get(key) {
            Some(value) => Some(value.clone()),
            None => default_value,
        };
        Ok(value)
    }

    pub fn keys(&self) -> Result<Vec<String>, Error> {
        let mut vec = vec![];
        let cache = self.state.cache.lock()?;
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

    pub fn on_update(&mut self) -> Result<Receiver<Cache>, Error> {
        let (tx, rx) = mpsc::channel();
        let mut subscribers = self.state.subscribers.lock()?;
        subscribers.push(tx);
        Ok(rx)
    }

    pub fn put(&mut self, key: &str, value: &str) -> Result<(), Error> {
        let args = vec![key.to_string(), value.to_string()];
        let _ = self.send(Format::Json, "put", &args)?;
        Ok(())
    }

    pub fn put_property(
        &mut self,
        node_id: &str,
        context_id: &str,
        profile: &str,
        property: &str,
        value: &str,
    ) -> Result<(), Error> {
        let system_id = system::get_system_id()?;
        let key =
            format!("cns/{system_id}/nodes/{node_id}/contexts/{context_id}/provider/{profile}/properties/{property}");
        self.put(&key, value)
    }

    pub fn send(&mut self, format: Format, cmd: &str, args: &[String]) -> Result<u64, Error> {
        let mut cmd = cmd.to_string();
        for arg in args {
            cmd = format!("{cmd} \"{arg}\"");
        }

        let mut msg = HashMap::new();
        let transaction_id = self.state.next_transaction_id.fetch_add(1, Ordering::SeqCst);
        msg.insert("format".to_string(), Value::String(format.to_string()));
        msg.insert("transaction".to_string(), Value::from(transaction_id));
        msg.insert("command".to_string(), Value::String(cmd));

        match format {
            Format::Json => {
                let msg_as_json = serde_json::to_string(&msg)?;
                let message = Message::text(msg_as_json);
                self.send_message(message)?;
                {
                    let mut requests = self.state.requests.lock()?;
                    requests.insert(transaction_id, None);
                }
                Ok(transaction_id)
            }
        }
    }

    fn send_message(&mut self, message: Message) -> Result<(), Error> {
        let mut socket = self.state.socket.lock()?;
        socket.send(message)?;
        Ok(())
    }

    pub fn stats(&self) -> Result<Stats, Error> {
        let cache = self.state.cache.lock()?;
        Ok(cache.stats.clone())
    }

    pub fn system(&mut self) -> Result<Arc<System>, Error> {
        let id = system::get_system_id()?;
        let name = hostname::get()?.to_str().unwrap().to_string();
        let args = vec![id.to_string(), name.to_string()];
        let transaction = self.send(Format::Json, "systems", &args)?;
        let _response = self.wait_for_response(transaction, Duration::from_secs(DEFAULT_TIMEOUT_SECS))?;
        Ok(Arc::new(System::new(self.clone(), id)))
    }

    pub fn version(&self) -> Result<String, Error> {
        let cache = self.state.cache.lock()?;
        Ok(cache.version.clone())
    }

    pub fn wait_for_open(&self, timeout: Duration) -> Result<(), Error> {
        let start_time = SystemTime::now();
        let sleep_for = Duration::from_millis(100);
        while SystemTime::now().duration_since(start_time)? < timeout {
            {
                let cache = self.state.cache.lock()?;
                if !cache.version.is_empty() {
                    return Ok(());
                }
            }
            std::thread::sleep(sleep_for);
        }
        Err(Error::Timeout("Timed out waiting for open".to_string()))
    }

    pub(crate) fn wait_for_response(&self, transaction: u64, timeout: Duration) -> Result<Response, Error> {
        let start_time = SystemTime::now();
        let sleep_for = Duration::from_millis(100);
        while SystemTime::now().duration_since(start_time)? < timeout {
            {
                let requests = self.state.requests.lock()?;
                let response = match requests.get(&transaction) {
                    None => return Err(Error::Default("No such transaction".to_string())),
                    Some(response) => response.clone(),
                };
                if let Some(response) = response {
                    match response.error.as_ref() {
                        None => return Ok(response.clone()),
                        Some(error_msg) => return Err(Error::Default(error_msg.clone())),
                    }
                }
            }
            std::thread::sleep(sleep_for);
        }
        Err(Error::Timeout("Timed out waiting for response".to_string()))
    }
}
