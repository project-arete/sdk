use super::{Client, Context, DEFAULT_TIMEOUT_SECS, Error, System};
use crate::client::Format;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct Node {
    client: Client,
    pub(crate) system: System,
    pub(crate) id: String,
}

impl Node {
    pub(crate) fn new(client: Client, system: System, id: String) -> Self {
        Self { client, system, id }
    }

    pub fn context(&self, id: &str, name: &str) -> Result<Arc<Context>, Error> {
        let args = vec![
            self.system.id.to_string(),
            self.id.clone(),
            id.to_string(),
            name.to_string(),
        ];
        let mut client = self.client.clone();
        let transaction = client.send(Format::Json, "contexts", &args)?;
        let _res = client.wait_for_response(transaction, Duration::from_secs(DEFAULT_TIMEOUT_SECS))?;
        Ok(Arc::new(Context::new(client, self.clone(), id.to_string())))
    }
}
