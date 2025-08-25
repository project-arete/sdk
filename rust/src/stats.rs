use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
pub enum ConnectionState {
    #[serde(rename = "offline")]
    #[default]
    Offline,

    #[serde(rename = "online")]
    Online,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Stats {
    pub started: String,
    pub reads: u32,
    pub writes: u32,
    pub updates: u32,
    pub errors: u32,
    pub connection: ConnectionState,
}
