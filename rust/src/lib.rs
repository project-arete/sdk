mod connection;
mod error;

use tungstenite::handshake::client::Response;
pub use connection::Connection;
pub use error::Error;

pub fn connect(url: &str) -> Result<(Connection, Response), Error> {
    let (socket, res) = tungstenite::connect(url)?;
    let connection = Connection::new(socket);
    Ok((connection, res))
}
