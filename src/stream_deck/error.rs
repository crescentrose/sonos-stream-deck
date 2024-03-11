use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

use super::SendEvent;

#[derive(Debug, Error)]
pub enum StreamDeckError {
    #[error("can't recognize received event")]
    UnrecognizedEvent,
    #[error("can't parse received json")]
    MalformedJson(#[from] serde_json::Error),
    #[error("send error")]
    SendError(#[from] SendError<SendEvent>),
    #[error("missing data in event")]
    MissingData,
    #[error("action does not exist")]
    UnknownAction,
    #[error("handler failed: {}", .0)]
    HandlerFailed(String),
    #[error("error reading from websockets")]
    ReadError(#[from] tokio_tungstenite::tungstenite::Error),
}
