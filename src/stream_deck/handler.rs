use log::error;
use serde::de::DeserializeOwned;
use tokio::sync::mpsc;

use super::{error::StreamDeckError, ReceiveEvent, SendEvent};

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
        self.chan
            .send(event)
            .await
            .map_err(StreamDeckError::SendError)
    }

    // Panics on purpose (if we can't log something has gone horribly wrong.)
    pub async fn log(&self, msg: &str) {
        self.send(SendEvent::Log {
            message: String::from(msg),
        })
        .await
        .unwrap();
    }

    pub async fn handle<'a, Actions: DeserializeOwned>(
        &self,
        event: &ReceiveEvent<Actions>,
        handler: &impl Handler<Actions>,
    ) -> Result<(), StreamDeckError> {
        handler.handle(self, event).await
    }

    pub async fn ingest<'a, Actions: DeserializeOwned>(
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
        .send(SendEvent::Register {
            uuid: uuid.to_string(),
        })
        .await
        .unwrap();
    connection.log("(ﾉ>ω<)ﾉ :｡･:*:･ﾟ’★,｡･:*:･ﾟ’☆").await;
    connection
}
