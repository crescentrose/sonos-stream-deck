use futures::{SinkExt, StreamExt};
use log::{debug, error, info, warn};
use std::{env, time::Duration};
use stream_deck::{Action, EventError, Handler};
use tokio::sync::mpsc;
use zone::Zone;

use crate::stream_deck::{ReceiveEvent, SendEvent};

pub mod error;
pub mod services;
mod stream_deck;
pub mod zone;

struct SonosHandler {
    zone: Option<Zone>,
    stream_deck: mpsc::Sender<SendEvent>,
}

impl Handler for SonosHandler {
    async fn on_key_up(&self, action: &Action, _context: &str) -> Result<(), EventError> {
        if let Some(zone) = &self.zone {
            match action {
                Action::PlayPause => zone
                    .play_pause()
                    .await
                    .map_err(|e| EventError::HandlerFailed(e.to_string()))?,
            }
        } else {
            warn!("no zone found");
        }

        self.stream_deck
            .send(SendEvent::Log {
                message: "complete key up".to_string(),
            })
            .await
            .unwrap();

        Ok(())
    }
}

impl SonosHandler {
    async fn new(stream_deck: mpsc::Sender<SendEvent>, uuid: &String) -> Self {
        stream_deck
            .send(SendEvent::Register { uuid: uuid.clone() })
            .await
            .unwrap();
        stream_deck
            .send(SendEvent::Log {
                message: "initializing sonos handler".to_string(),
            })
            .await
            .unwrap();
        let zones = Zone::get_zones(Duration::from_secs(5)).await;
        if let Ok(mut zones) = zones {
            if let Some(found_zone) = zones.pop() {
                info!("found zone: {}", found_zone.name());
                return Self {
                    zone: Some(found_zone),
                    stream_deck,
                };
            }
        }
        warn!("no sonos zones found");
        return Self {
            zone: None,
            stream_deck,
        };
    }
}

#[tokio::main(flavor = "current_thread")] // no need for multithreading, keep it simple
async fn main() {
    // Log directly to a file as we can't read stdout/stderr from the Stream Deck app
    // log4rs.yml should be in the *.sdPlugin directory
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    log_panics::init();

    info!("starting plugin");
    debug!("pid: {}", std::process::id()); // useful to `kill` the process so the SD app restarts it

    // really quick and dirty argument parsing - should probably use `clap` later
    let args: Vec<String> = env::args().collect();
    let port: u16 = args[2].parse().unwrap();
    let uuid = &args[4];
    let register_event = &args[6];
    debug!("port: {port}, uuid: {uuid}, registerEvent: {register_event}");

    // set up websockets client
    let (conn, _) = tokio_tungstenite::connect_async(format!("ws://localhost:{}", port))
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

    let handler = SonosHandler::new(send_tx, uuid).await;
    let handler_future = handler.handle(&mut recv_rx);

    // start the send and handler tasks and wait until they are done
    // the handler shuts down when the Stream Deck closes the WebSockets connection
    futures::pin_mut!(sender, reader, handler_future);
    futures::join!(sender, reader, handler_future);

    info!("plugin shutting down -- goodbye!");
}
