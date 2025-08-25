mod connection;

use anyhow::Result;
use tungstenite::handshake::client::Response;
pub use connection::Connection;

pub fn connect(url: &str) -> Result<(Connection, Response)> {
    let (socket, res) = tungstenite::connect(url)?;
    let connection = Connection::new(socket);
    Ok((connection, res))
}
