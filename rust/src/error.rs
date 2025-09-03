use std::{sync::PoisonError, time::SystemTimeError};
use strum_macros::Display;

#[derive(Debug, Display)]
pub enum Error {
    #[strum(to_string = "Default Error: {0}")]
    Default(String),

    #[strum(to_string = "Io Error: {0}")]
    Io(String),

    #[strum(to_string = "Lock Error: {0}")]
    Lock(String),

    #[strum(to_string = "Serialization Error: {0}")]
    Serialization(String),

    #[strum(to_string = "Timeout Error: {0}")]
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

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
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

#[cfg(test)]
mod tests {
    use crate::Error;

    #[test]
    fn can_display() {
        let e = Error::Default("the message".to_string());
        assert_eq!(e.to_string(), "Default Error: the message");
    }
}
