use super::{Client, System};

pub struct Node {
    #[allow(unused)]
    client: Client,
    #[allow(unused)]
    system: System,
    #[allow(unused)]
    id: String,
}

impl Node {
    pub(crate) fn new(client: Client, system: System, id: String) -> Node {
        Self { client, system, id }
    }
}
