use super::{Error, Stats};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};
use std::time::{Duration, SystemTime};
use strum_macros::{AsRefStr, Display};
use tungstenite::{Message, WebSocket, stream::MaybeTlsStream};

#[derive(AsRefStr, Clone, Debug, Display)]
pub enum Format {
    #[strum(serialize = "json")]
    Json,
}

pub struct Connection {
    socket: Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>,
    next_transaction_id: AtomicU64,
    cache: Arc<Mutex<Cache>>,
}

#[derive(Debug, Default, Deserialize)]
struct Cache {
    version: String,
    stats: Stats,
    keys: HashMap<String, Value>,
}

impl Connection {
    pub(crate) fn new(socket: Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>) -> Self {
        let next_transaction_id = AtomicU64::default();
        let cache = Arc::new(Mutex::new(Cache::default()));

        // Spawn a thread to handle incoming messages
        let socket_2 = socket.clone();
        let cache_2 = cache.clone();
        std::thread::spawn(move || {
            loop {
                let maybe_message = {
                    if let Ok(mut socket) = socket_2.lock() {
                        Some(socket.read().unwrap())
                    } else {
                        None
                    }
                };
                let message = match maybe_message {
                    Some(message) => message,
                    _ => continue,
                };
                if let Message::Text(ref message) = message {
                    let payload: Cache = serde_json::from_slice(message.as_bytes()).unwrap();
                    if let Ok(mut cache) = cache_2.lock() {
                        Self::merge(&mut cache, &payload);
                    }
                }
            }
        });

        Self {
            socket,
            next_transaction_id,
            cache,
        }
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

    fn merge(target: &mut Cache, source: &Cache) {
        target.stats = source.stats.clone();
        target.version = source.version.clone();
        for (k, v) in source.keys.iter() {
            target.keys.insert(k.to_string(), v.clone());
        }
    }

    pub fn send(&mut self, format: Format, cmd: &str, args: &[&str]) -> Result<(), Error> {
        let mut cmd = cmd.to_string();
        for arg in args {
            cmd = format!("{cmd} \"{arg}\"");
        }

        let mut msg = HashMap::new();
        let transaction_id = self.next_transaction_id.fetch_add(1, Ordering::SeqCst);
        msg.insert("format".to_string(), Value::String(format.to_string()));
        msg.insert("transaction".to_string(), Value::from(transaction_id));
        msg.insert("command".to_string(), Value::String(cmd));

        let msg_as_json = serde_json::to_string(&msg)?;
        let message = Message::text(msg_as_json);
        self.send_message(message)
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
}
