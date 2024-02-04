use std::any::Any;

use log::error;
use serde_json::{json, Value};
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

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

#[derive(Debug, Error)]
pub enum EventError {
    #[error("can't recognize received event")]
    UnrecognizedEvent,
    #[error("can't parse received json")]
    MalformedJson(#[from] serde_json::Error),
    #[error("missing data in event")]
    MissingData,
    #[error("action does not exist")]
    UnknownAction,
    #[error("handler failed: {}", .0)]
    HandlerFailed(String),
}

pub trait Handler {
    async fn handle(&self, incoming: &mut mpsc::Receiver<ReceiveEvent>) {
        while let Some(event) = incoming.recv().await {
            let res = match event {
                ReceiveEvent::KeyUp {
                    ref action,
                    ref context,
                } => self.on_key_up(action, context).await,
                _ => Ok(()),
            };

            if let Err(e) = res {
                error!("error handling event {:?}: {:?}", event.type_id(), e)
            }
        }
    }

    async fn on_key_up(&self, action: &Action, context: &str) -> Result<(), EventError>;
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
    pub fn from_message(event: &str) -> Result<ReceiveEvent, EventError> {
        use EventError::*;
        use ReceiveEvent::*;

        let event: Value = serde_json::from_str(event)?;
        let event_type = event.strict_get("event")?;

        match event_type {
            "keyDown" => Ok(KeyDown {
                action: Action::from_identifier(event.strict_get("action")?)?,
                context: event.strict_get("context")?.to_string(),
            }),
            "keyUp" => Ok(KeyUp {
                action: Action::from_identifier(event.strict_get("action")?)?,
                context: event.strict_get("context")?.to_string(),
            }),
            _ => Err(UnrecognizedEvent),
        }
    }
}

trait StrictGet {
    fn strict_get(&self, key: &str) -> Result<&str, EventError>;
}

impl StrictGet for Value {
    fn strict_get(&self, key: &str) -> Result<&str, EventError> {
        self.get(key)
            .ok_or(EventError::MissingData)?
            .as_str()
            .ok_or(EventError::MissingData)
    }
}

impl Action {
    pub fn from_identifier(identifier: &str) -> Result<Self, EventError> {
        match identifier {
            "sh.viora.controller-for-sonos.play-pause" => Ok(Action::PlayPause),
            _ => Err(EventError::UnknownAction),
        }
    }
}
