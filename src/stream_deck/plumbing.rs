use futures::{SinkExt, StreamExt};
use log::{debug, error};
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;

use crate::stream_deck::handler;

use super::{handler::Handler, ReceiveEvent, SendEvent};

pub async fn run<H: Handler>(port: u16, uuid: &String, hndlr: H) {
    // set up websockets client
    let (conn, _) = connect_async(format!("ws://localhost:{}", port))
        .await
        .expect("connection failed");
    debug!("connection successful");

    // this lets us have separate tasks to send and receive events
    let (mut write, read) = conn.split();

    // set up channels to send and receive events
    let (send_tx, mut send_rx) = mpsc::channel::<SendEvent>(32);
    let (recv_tx, mut recv_rx) = mpsc::channel::<ReceiveEvent>(32);

    let sender = async {
        while let Some(event) = send_rx.recv().await {
            match write.send(event.clone().into()).await {
                Ok(_) => debug!("sent event: {:?}", event),
                Err(e) => error!("error sending event: {:?}", e),
            }
        }
    };

    let reader = {
        read.for_each(|message| async {
            let data = message.unwrap().into_data();
            let msg = String::from_utf8_lossy(&data);
            debug!("received message: {:?}", msg);
            match ReceiveEvent::from_message(&msg) {
                Ok(event) => recv_tx.send(event).await.unwrap(),
                Err(e) => error!("error receiving event: {:?}", e),
            };
        })
    };

    let connection = handler::initialize(send_tx, uuid).await;
    let handler = connection.ingest(&mut recv_rx, hndlr);

    // start the send and handler tasks and wait until they are done
    // the handler shuts down when the Stream Deck closes the WebSockets connection
    futures::pin_mut!(sender, reader, handler);
    futures::join!(sender, reader, handler);
}
