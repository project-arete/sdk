use super::{Client, Context, Error};
use crate::consumer::ChangeEvent;
use regex::Regex;
use serde_json::Value;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;

pub struct Provider {
    client: Client,
    context: Context,
    pub profile: String,
}

impl Provider {
    pub(crate) fn new(client: Client, context: Context, profile: String) -> Self {
        Self {
            client,
            context,
            profile,
        }
    }

    fn profile_key_prefix(&self) -> String {
        let system_id = self.context.node.system.id.to_string();
        let node_id = &self.context.node.id;
        let context_id = &self.context.id;
        let profile = &self.profile;
        format!("cns/{system_id}/nodes/{node_id}/contexts/{context_id}/provider/{profile}/")
    }

    fn property_key(&self, property: &str) -> String {
        let profile_key_prefix = self.profile_key_prefix();
        format!("{profile_key_prefix}properties/{property}")
    }

    pub fn get(&self, property: &str, default_value: Option<Value>) -> Result<Option<Value>, Error> {
        let key = self.property_key(property);
        self.client.get(&key, default_value)
    }

    pub fn put(&self, property: &str, value: &str) -> Result<(), Error> {
        let key = self.property_key(property);
        let mut client = self.client.clone();
        client.put(&key, value)
    }

    pub fn watch(&self) -> Result<Receiver<ChangeEvent>, Error> {
        let key_prefix = self.profile_key_prefix();
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
                        let change_event = ChangeEvent {
                            connection,
                            property,
                            value,
                        };
                        tx.send(change_event).unwrap();
                    };
                }
            }
        });
        Ok(rx)
    }
}
