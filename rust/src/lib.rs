mod connection;
mod error;
mod stats;

pub use connection::Connection;
pub use error::Error;
pub use stats::Stats;
use std::sync::{Arc, Mutex};
use tungstenite::handshake::client::Response;

pub fn connect(url: &str) -> Result<(Connection, Response), Error> {
    let (socket, res) = tungstenite::connect(url)?;
    let connection = Connection::new(Arc::new(Mutex::new(socket)));
    Ok((connection, res))
}
