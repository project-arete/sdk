mod connection;
mod error;
mod stats;

pub use connection::Connection;
pub use error::Error;
pub use stats::Stats;
use std::sync::{Arc, Mutex};
use tungstenite::handshake::client::Response;
use tungstenite::stream::MaybeTlsStream;

pub fn connect(url: &str) -> Result<(Connection, Response), Error> {
    // Connect
    let (mut socket, res) = tungstenite::connect(url)?;

    // Configure non-blocking reads
    match socket.get_mut() {
        MaybeTlsStream::Plain(stream) => stream.set_nonblocking(true)?,
        MaybeTlsStream::Rustls(stream) => stream.get_mut().set_nonblocking(true)?,
        _ => {}
    }

    // Respond
    let connection = Connection::new(Arc::new(Mutex::new(socket)));
    Ok((connection, res))
}
