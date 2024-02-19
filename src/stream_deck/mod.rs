pub mod error;
pub mod handler;
pub mod plumbing;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;

use self::error::StreamDeckError;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum SendEvent {
    #[serde(alias = "registerPlugin")]
    Register { uuid: String },
    #[serde(alias = "logMessage")]
    Log {
        #[serde(with = "payload")]
        message: String,
    },
}

#[non_exhaustive]
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum ReceiveEvent<Action> {
    DidReceiveSettings {
        action: Action,
        context: String,
        device: String,
        payload: Value,
    },
    DidReceiveGlobalSettings {
        payload: Value,
    },
    DidReceiveDeepLink {
        payload: Value,
    },
    KeyDown {
        action: Action,
        context: String,
        payload: Value,
    },
    KeyUp {
        action: Action,
        context: String,
        payload: Value,
    },
    WillAppear {
        action: Action,
        context: String,
        device: String,
        payload: Value,
    },
    WillDisappear {
        action: Action,
        context: String,
        device: String,
        payload: Value,
    },
    DeviceDidConnect {
        device: String,
    },
    DeviceDidDisconnect {
        device: String,
    },
    ApplicationDidLaunch {
        payload: Value,
    },
    ApplicationDidTerminate {
        payload: Value,
    },
    PropertyInspectorDidAppear,
    PropertyInspectorDidDisappear,
    SystemDidWakeUp,
    SendToPlugin {
        action: Action,
        context: String,
        payload: Value,
    },
}

impl TryFrom<SendEvent> for Message {
    type Error = StreamDeckError;

    fn try_from(value: SendEvent) -> Result<Self, Self::Error> {
        let msg = serde_json::to_string(&value).unwrap();

        if value.is_binary() {
            Ok(Message::Binary(msg.into_bytes()))
        } else {
            Ok(Message::Text(msg))
        }
    }
}

impl SendEvent {
    pub fn is_binary(&self) -> bool {
        matches!(self, SendEvent::Register { uuid: _ })
    }
}

mod payload {
    use serde::{ser::SerializeStruct, Deserializer, Serializer};

    pub(crate) fn deserialize<'de, D>(de: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }

    pub(crate) fn serialize<S>(value: &str, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut payload = ser.serialize_struct("payload", 1)?;
        payload.serialize_field("message", value)?;
        payload.end()
    }
}
