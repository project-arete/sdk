use std::sync::Arc;
use std::time::Duration;
use crate::client::Format;
use super::{Provider, Client, Error, Node, DEFAULT_TIMEOUT_SECS, Consumer};

#[derive(Clone)]
pub struct Context {
    client: Client,
    node: Node,
    id: String,
}

impl Context {
    pub(crate) fn new(client: Client, node: Node, id: String) -> Context {
        Self { client, node, id }
    }

    pub fn consumer(&self, profile: &str) -> Result<Arc<Consumer>, Error> {
        let args = vec![
            self.node.system.id.to_string(),
            self.node.id.clone(),
            self.id.to_string(),
            profile.to_string(),
        ];
        let mut client = self.client.clone();
        let transaction = client.send(Format::Json, "consumers", &args)?;
        let _res = client
            .wait_for_response(transaction, Duration::from_secs(DEFAULT_TIMEOUT_SECS))?;
        Ok(Arc::new(Consumer::new(client, self.clone(), profile.to_string())))
    }

    pub fn provider(&self, profile: &str) -> Result<Arc<Provider>, Error> {
        let args = vec![
            self.node.system.id.to_string(),
            self.node.id.clone(),
            self.id.to_string(),
            profile.to_string(),
        ];
        let mut client = self.client.clone();
        let transaction = client.send(Format::Json, "providers", &args)?;
        let _res = client
            .wait_for_response(transaction, Duration::from_secs(DEFAULT_TIMEOUT_SECS))?;
        Ok(Arc::new(Provider::new(client, self.clone(), profile.to_string())))
    }
}
