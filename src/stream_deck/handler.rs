use log::{error, info};
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use tokio::sync::mpsc;

use super::{error::StreamDeckError, payload, ReceiveEvent, SendEvent};

pub struct Connection {
    chan: mpsc::Sender<SendEvent>,
}

pub trait Handler<Actions> {
    async fn handle(
        &self,
        connection: &Connection,
        event: &ReceiveEvent<Actions>,
    ) -> Result<(), StreamDeckError>;
}

impl Connection {
    pub async fn send(&self, event: SendEvent) -> Result<(), StreamDeckError> {
        info!("sending event: {event:?}");
        self.chan
            .send(event)
            .await
            .map_err(StreamDeckError::SendError)
    }

    // Panics on purpose (if we can't log something has gone horribly wrong.)
    pub async fn log(&self, msg: &str) {
        self.send(SendEvent::Log {
            payload: payload::Log {
                message: msg.to_string(),
            },
        })
        .await
        .unwrap();
    }

    pub async fn handle<'a, Actions>(
        &self,
        event: &ReceiveEvent<Actions>,
        handler: &impl Handler<Actions>,
    ) -> Result<(), StreamDeckError>
    where
        Actions: DeserializeOwned + Debug,
    {
        info!("handling {event:?}");
        handler.handle(self, event).await
    }

    pub async fn ingest<'a, Actions: DeserializeOwned + Debug>(
        &self,
        incoming: &mut mpsc::Receiver<ReceiveEvent<Actions>>,
        handler: impl Handler<Actions>,
    ) {
        while let Some(event) = incoming.recv().await {
            let res = self.handle(&event, &handler).await;
            if let Err(e) = res {
                error!("error handling event: {:?}", e)
            }
        }
    }
}

pub async fn initialize(chan: mpsc::Sender<SendEvent>, uuid: &str) -> Connection {
    let connection = Connection { chan };
    connection
        .send(SendEvent::RegisterPlugin {
            uuid: uuid.to_string(),
        })
        .await
        .unwrap();
    connection.log("(ﾉ>ω<)ﾉ :｡･:*:･ﾟ’★,｡･:*:･ﾟ’☆").await;
    connection
}
