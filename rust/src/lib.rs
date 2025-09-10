mod client;
mod error;
mod stats;
mod system;
mod cache;

pub use client::Client;
pub use cache::Cache;
pub use error::Error;
pub use stats::Stats;
use std::sync::{Arc, Mutex};
use tungstenite::{handshake::client::Response, stream::MaybeTlsStream};

pub fn connect(url: &str) -> Result<(Client, Response), Error> {
    // Connect
    let (mut socket, res) = tungstenite::connect(url)?;

    // Configure non-blocking reads
    match socket.get_mut() {
        MaybeTlsStream::Plain(stream) => stream.set_nonblocking(true)?,
        MaybeTlsStream::Rustls(stream) => stream.get_mut().set_nonblocking(true)?,
        _ => {}
    }

    // Respond
    let client = Client::new(Arc::new(Mutex::new(socket)));
    Ok((client, res))
}
