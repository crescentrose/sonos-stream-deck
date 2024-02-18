pub mod error;
pub mod handler;
pub mod plumbing;

use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::Message;

use self::error::StreamDeckError;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum SendEvent {
    Register { uuid: String },
    Log { message: String },
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum ReceiveEvent {
    KeyDown { action: Action, context: String },
    KeyUp { action: Action, context: String },
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    PlayPause,
}

impl From<SendEvent> for Message {
    fn from(value: SendEvent) -> Self {
        let msg = match value {
            SendEvent::Register { ref uuid } => json!({"event": "registerPlugin", "uuid": uuid}),
            SendEvent::Log { ref message } => {
                json!({"event": "logMessage", "payload": {"message": message}})
            }
        }
        .to_string();

        if value.is_binary() {
            Message::Binary(msg.into_bytes())
        } else {
            Message::Text(msg)
        }
    }
}

impl SendEvent {
    pub fn is_binary(&self) -> bool {
        matches!(self, SendEvent::Register { uuid: _ })
    }
}

impl ReceiveEvent {
    pub fn from_message(event: &str) -> Result<ReceiveEvent, StreamDeckError> {
        use ReceiveEvent::*;
        use StreamDeckError::*;

        let event: Value = serde_json::from_str(event)?;
        let event_type = event.try_get("event")?;

        match event_type {
            "keyDown" => Ok(KeyDown {
                action: Action::try_from(event.try_get("action")?)?,
                context: event.try_get("context")?.to_string(),
            }),
            "keyUp" => Ok(KeyUp {
                action: Action::try_from(event.try_get("action")?)?,
                context: event.try_get("context")?.to_string(),
            }),
            _ => Err(UnrecognizedEvent),
        }
    }
}

trait TryGet {
    fn try_get(&self, key: &str) -> Result<&str, StreamDeckError>;
}

impl TryGet for Value {
    fn try_get(&self, key: &str) -> Result<&str, StreamDeckError> {
        self.get(key)
            .ok_or(StreamDeckError::MissingData)?
            .as_str()
            .ok_or(StreamDeckError::MissingData)
    }
}

impl TryFrom<&str> for Action {
    type Error = StreamDeckError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "sh.viora.controller-for-sonos.play-pause" => Ok(Action::PlayPause),
            _ => Err(StreamDeckError::UnknownAction),
        }
    }
}
