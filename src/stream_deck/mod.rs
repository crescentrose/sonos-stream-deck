pub mod error;
pub mod handler;
pub mod plumbing;

use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;

use self::error::StreamDeckError;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum SendEvent {
    #[serde(alias = "register")]
    RegisterPlugin { uuid: String },
    #[serde(alias = "logMessage")]
    Log {
        #[serde(flatten)]
        payload: payload::Log,
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
        payload: payload::KeyPress,
    },
    KeyUp {
        action: Action,
        context: String,
        payload: payload::KeyPress,
    },
    WillAppear {
        action: Action,
        context: String,
        device: String,
        payload: payload::Presence,
    },
    WillDisappear {
        action: Action,
        context: String,
        device: String,
        payload: payload::Presence,
    },
    DeviceDidConnect {
        device: String,
    },
    DeviceDidDisconnect {
        device: String,
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
            debug!("sending bytes: {msg}");
            Ok(Message::Binary(msg.into_bytes()))
        } else {
            debug!("sending json: {msg}");
            Ok(Message::Text(msg))
        }
    }
}

impl SendEvent {
    pub fn is_binary(&self) -> bool {
        matches!(self, SendEvent::RegisterPlugin { uuid: _ })
    }
}

mod payload {
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Debug, Copy, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Coordinates {
        pub column: i32,
        pub row: i32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Log {
        pub message: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct KeyPress {
        pub settings: Value,
        pub coordinates: Coordinates,
        pub state: Option<i32>,
        pub user_desired_state: Option<i32>,
        pub is_in_multi_action: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Presence {
        pub settings: Value,
        pub coordinates: Coordinates,
        pub controller: String,
        pub state: Option<i32>,
        pub is_in_multi_action: bool,
    }
}

macro_rules! action_names {
    ($actions:ident => { $($name:expr => $variant:expr) , + }) => {
        use serde::{de, Deserialize};

        impl<'de> Deserialize<'de> for $actions {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                match String::deserialize(deserializer)?.as_str() {
                    $(
                        $name => Ok($variant),
                    )+
                    val => Err(de::Error::invalid_value(
                        de::Unexpected::Str(val),
                        &"a valid action",
                    )),
                }
            }
        }
    };
}
