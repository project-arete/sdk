use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use serde_json::Value;
use strum_macros::{AsRefStr, Display};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};
use super::Error;

#[derive(AsRefStr, Clone, Debug, Display)]
pub enum Format {
    #[strum(serialize = "json")]
    Json,
}

pub struct Connection {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    next_transaction_id: AtomicU64,
    cache: Mutex<HashMap<String, Value>>,
}

impl Connection {
    pub(crate) fn new(socket: WebSocket<MaybeTlsStream<TcpStream>>) -> Self {
        let next_transaction_id = AtomicU64::default();
        let cache = Mutex::new(HashMap::new());
        Self{socket, next_transaction_id, cache}
    }

    pub fn get(&self, key: &str, default_value: Option<Value>) -> Result<Option<Value>, Error> {
        let cache = self.cache.lock().map_err(|e| Error::Lock(e.to_string()))?;
        let value = match cache.get(key) {
            Some(value) => Some(value.clone()),
            None => default_value,
        };
        Ok(value)
    }

    pub fn keys(&self) -> Result<Vec<String>, Error> {
        let mut vec = vec![];
        let cache = self.cache.lock().map_err(|e| Error::Lock(e.to_string()))?;
        for key in cache.keys() {
            vec.push(key.clone());
        }
        Ok(vec)
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
        let message= Message::text(msg_as_json);
        self.send_message(message)
    }

    fn send_message(&mut self, message: Message) -> Result<(), Error> {
        self.socket.send(message)?;
        Ok(())
    }
}
