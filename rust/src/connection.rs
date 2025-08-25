use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::atomic::{AtomicU64, Ordering};
use serde_json::Value;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

pub struct Connection {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    next_transaction_id: AtomicU64,
}

impl Connection {
    pub(crate) fn new(socket: WebSocket<MaybeTlsStream<TcpStream>>) -> Self {
        let next_transaction_id = AtomicU64::default();
        Self{socket, next_transaction_id}
    }

    pub fn send(&mut self, format: &str, cmd: &str, args: &[&str]) -> anyhow::Result<()> {
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

    fn send_message(&mut self, message: Message) -> anyhow::Result<()> {
        self.socket.send(message)?;
        Ok(())
    }
}
