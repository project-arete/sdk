use std::net::TcpStream;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

pub struct Connection {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
}

impl Connection {
    pub(crate) fn new(socket: WebSocket<MaybeTlsStream<TcpStream>>) -> Self {
        Self{socket}
    }

    pub fn send(&mut self, message: Message) -> anyhow::Result<()> {
        self.socket.send(message)?;
        Ok(())
    }
}
