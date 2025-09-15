use super::{Client, Context, Error};

pub struct Provider {
    client: Client,
    context: Context,
    profile: String,
}

impl Provider {
    pub(crate) fn new(client: Client, context: Context, profile: String) -> Self {
        Self {
            client,
            context,
            profile,
        }
    }

    pub fn put(&self, property: &str, value: &str) -> Result<(), Error> {
        let system_id = self.context.node.system.id.to_string();
        let node_id = &self.context.node.id;
        let context_id = &self.context.id;
        let profile = &self.profile;
        let key =
            format!("cns/{system_id}/nodes/{node_id}/contexts/{context_id}/provider/{profile}/properties/{property}");
        let mut client = self.client.clone();
        client.put(&key, value)
    }
}
