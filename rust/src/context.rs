use super::{Client, Node};

pub struct Context {
    #[allow(unused)]
    client: Client,
    #[allow(unused)]
    node: Node,
    #[allow(unused)]
    id: String,
}

impl Context {
    pub(crate) fn new(client: Client, node: Node, id: String) -> Context {
        Self { client, node, id }
    }
}
