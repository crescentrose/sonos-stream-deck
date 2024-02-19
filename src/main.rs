use crate::stream_deck::error::StreamDeckError;
use crate::stream_deck::ReceiveEvent;
use log::{debug, info, warn};
use serde::Deserialize;
use sonos::Zone;
use std::{env, time::Duration};
use stream_deck::handler::{Connection, Handler};

pub mod sonos;
pub(crate) mod stream_deck;

struct SonosHandler {
    zone: Option<Zone>,
}

// TODO: Extract
#[derive(Debug, Clone, Copy, Deserialize)]
pub enum Action {
    PlayPause,
}

impl Handler<Action> for SonosHandler {
    async fn handle(
        &self,
        _connection: &Connection,
        event: &ReceiveEvent<Action>,
    ) -> Result<(), StreamDeckError> {
        match event {
            ReceiveEvent::KeyUp { action, .. } => self.action(action).await,
            _ => Ok(()),
        }
    }
}

impl SonosHandler {
    async fn new() -> Self {
        let zones = Zone::get_zones(Duration::from_secs(5)).await;
        if let Ok(mut zones) = zones {
            if let Some(found_zone) = zones.pop() {
                info!("found zone: {}", found_zone.name());
                return Self {
                    zone: Some(found_zone),
                };
            }
        }
        warn!("no sonos zones found");
        return Self { zone: None };
    }

    async fn action(&self, action: &Action) -> Result<(), StreamDeckError> {
        match action {
            Action::PlayPause => self.play_pause().await,
        }
    }

    async fn play_pause(&self) -> Result<(), StreamDeckError> {
        if let Some(zone) = &self.zone {
            zone.play_pause()
                .await
                .map_err(|e| StreamDeckError::HandlerFailed(e.to_string()))
        } else {
            warn!("no zone detected");
            Ok(())
        }
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

    let handler = SonosHandler::new().await;
    stream_deck::plumbing::run(port, uuid, handler).await;

    info!("plugin shutting down -- goodbye!");
}
