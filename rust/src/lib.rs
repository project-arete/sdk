mod cache;
mod client;
mod consumer;
mod context;
mod error;
mod node;
mod provider;
mod stats;
mod system;

pub use cache::Cache;
pub use client::{Client, DEFAULT_TIMEOUT_SECS};
pub use consumer::Consumer;
pub use context::Context;
pub use error::Error;
pub use node::Node;
pub use provider::Provider;
pub use stats::Stats;
use std::sync::{Arc, Mutex};
pub use system::System;
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
