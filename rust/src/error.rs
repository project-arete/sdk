#[derive(Debug)]
pub enum Error {
    Default(String),
    Io(String),
    Lock(String),
    Serialization(String),
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Serialization(e.to_string())
    }
}

impl From<tungstenite::Error> for Error {
    fn from(e: tungstenite::Error) -> Self {
        Error::Io(e.to_string())
    }
}
