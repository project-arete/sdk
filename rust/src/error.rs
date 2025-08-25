use std::sync::PoisonError;
use std::time::SystemTimeError;

#[derive(Debug)]
pub enum Error {
    Default(String),
    Io(String),
    Lock(String),
    Serialization(String),
    Timeout(String),
}

impl<T> From<PoisonError<T>> for Error {
    fn from(e: PoisonError<T>) -> Self {
        Self::Lock(e.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Serialization(e.to_string())
    }
}

impl From<SystemTimeError> for Error {
    fn from(e: SystemTimeError) -> Self {
        Error::Default(e.to_string())
    }
}

impl From<tungstenite::Error> for Error {
    fn from(e: tungstenite::Error) -> Self {
        Error::Io(e.to_string())
    }
}
