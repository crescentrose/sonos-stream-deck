use std::fmt::Debug;

use futures::{SinkExt, StreamExt};
use log::{debug, error, info};
use serde::de::DeserializeOwned;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;

use crate::stream_deck::{error::StreamDeckError, handler};

use super::{handler::Handler, ReceiveEvent, SendEvent};

pub async fn run<H: Handler<Actions>, Actions: DeserializeOwned + Debug>(
    port: u16,
    uuid: &String,
    hndlr: H,
) {
    // set up websockets client
    let (conn, _) = connect_async(format!("ws://localhost:{}", port))
        .await
        .expect("connection failed");
    debug!("connection successful");

    // this lets us have separate tasks to send and receive events
    let (mut write, read) = conn.split();

    // set up channels to send and receive events
    let (send_tx, mut send_rx) = mpsc::channel::<SendEvent>(32);
    let (recv_tx, mut recv_rx) = mpsc::channel::<ReceiveEvent<Actions>>(32);

    let sender = async {
        while let Some(event) = send_rx.recv().await {
            match write.send(event.clone().try_into().unwrap()).await {
                Ok(_) => debug!("sent event: {:?}", event),
                Err(e) => error!("error sending event: {:?}", e),
            }
        }
    };

    let reader = {
        read.for_each(|message| async {
            let msg = message
                .map(|m| m.into_data())
                .map_err(StreamDeckError::ReadError)
                .and_then(|e| Ok(String::from_utf8(e).unwrap()));

            if let Ok(json) = msg {
                if json.parse::<i64>().is_ok() {
                    // sometimes we just get numbers from the stream deck, we ignore those
                    return;
                }
                debug!("received json: {:?}", json);
                let event = serde_json::from_str(&json).map_err(StreamDeckError::MalformedJson);
                info!("dispatching event: {:?}", event);

                match event {
                    Ok(event) => recv_tx.send(event).await.unwrap(),
                    Err(e) => error!("error processing event: {:?}", e),
                };
            }
        })
    };

    let connection = handler::initialize(send_tx, uuid).await;
    let handler = connection.ingest(&mut recv_rx, hndlr);

    // start the send and handler tasks and wait until they are done
    // the handler shuts down when the Stream Deck closes the WebSockets connection
    futures::pin_mut!(sender, reader, handler);
    futures::join!(sender, reader, handler);
}
