use super::{Client, Context};

pub struct Consumer {
    #[allow(unused)]
    client: Client,
    #[allow(unused)]
    context: Context,
    #[allow(unused)]
    profile: String,
}

impl Consumer {
    pub(crate) fn new(client: Client, context: Context, profile: String) -> Self {
        Self {
            client,
            context,
            profile,
        }
    }
}
