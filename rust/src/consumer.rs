use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use regex::Regex;
use serde_json::Value;
use super::{Client, Context, Error};

pub struct Consumer {
    #[allow(unused)]
    client: Client,
    #[allow(unused)]
    context: Context,
    #[allow(unused)]
    pub profile: String,
}

#[derive(Clone, Debug, Default)]
pub struct ChangeEvent {
    #[allow(unused)]
    pub connection: String,
    #[allow(unused)]
    pub property: String,
    #[allow(unused)]
    pub value: Value,
}

impl Consumer {
    pub(crate) fn new(client: Client, context: Context, profile: String) -> Self {
        Self {
            client,
            context,
            profile,
        }
    }

    pub fn watch(&self) -> Result<Receiver<ChangeEvent>, Error> {
        let system_id = self.context.node.system.id.to_string();
        let node_id = &self.context.node.id;
        let context_id = &self.context.id;
        let profile = &self.profile;
        let key_prefix =
            format!("cns/{system_id}/nodes/{node_id}/contexts/{context_id}/consumer/{profile}/");
        let upstream_rx = self.client.clone().on_update()?;
        let (tx, rx) = mpsc::channel();
        let re = Regex::new(r"connections/(\w+)/properties/(\w+)$")?;
        std::thread::spawn(move || {
            for event in upstream_rx {
                for (k, v) in event.keys.iter() {
                    if !k.starts_with(&key_prefix) {
                        continue;
                    }
                    if let Some(captures) = re.captures(k) {
                        let connection = captures[1].to_string();
                        let property = captures[2].to_string();
                        let value = v.clone();
                        let change_event = ChangeEvent { connection, property, value };
                        tx.send(change_event).unwrap();
                    };
                }
            }
        });
        Ok(rx)
    }

}
