mod error;
mod services;
mod zone;

use std::{sync::Arc, time::Duration};

use error::ControllerError;
use futures::{pin_mut, SinkExt, StreamExt};
use log::{debug, error, info};
use serde_json::{json, Value};
use tokio::{self, sync::Mutex};
use tokio_tungstenite::{self, tungstenite::Message};

use crate::zone::Zone;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ControllerError> {
    log4rs::init_file(
        format!(
            "/Users/{}/Code/sonos-controller/log4rs.yml",
            std::env::var("USER").unwrap()
        ),
        Default::default(),
    )
    .unwrap();
    log_panics::init();

    info!("starting server");
    debug!("pid: {}", std::process::id());

    let zone: Arc<Mutex<Option<Zone>>> = Arc::new(Mutex::new(None));

    let get_zone = async {
        let zones = Zone::get_zones(Duration::from_secs(5)).await;
        if let Ok(mut zones) = zones {
            if let Some(found_zone) = zones.pop() {
                info!("found zone: {}", found_zone.name());

                let mut zone = zone.lock().await;
                *zone = Some(found_zone);
            }
        }
    };

    let port: u16 = std::env::args()
        .nth(2)
        .expect("port not supplied")
        .parse()
        .expect("port not u16");
    debug!("received port: {}", port);

    let uuid: String = std::env::args().nth(4).expect("uuid not supplied");
    debug!("received uuid: {}", uuid);

    let event: String = std::env::args().nth(6).expect("event not supplied");
    debug!("received registerEvent: {}", event);

    let info: String = std::env::args().nth(8).expect("info not supplied");
    debug!("received info: {}", info);

    let (conn, _) = tokio_tungstenite::connect_async(format!("ws://localhost:{}", port))
        .await
        .expect("connection failed 😧");
    debug!("WebSocket handshake has been successfully completed 🥳");

    let (mut write, read) = conn.split();

    let read_ws = {
        read.for_each(|message| async {
            let data = message.unwrap().into_data();
            let msg = String::from_utf8_lossy(&data);
            debug!("received message: {:?}", msg);
            let event: Result<Value, serde_json::Error> = serde_json::from_str(&msg);
            if let Ok(res) = event {
                let typ = res.get("event");
                let act = res.get("action");
                match (typ, act) {
                    (Some(t), Some(a)) => {
                        info!("user action: {} (event: {})", t, a);
                        if t == "keyUp" && a == "sh.viora.controller-for-sonos.play-pause" {
                            let mut zone = zone.lock().await;
                            if let Some(z) = &mut *zone {
                                let _ = z
                                    .play_pause()
                                    .await
                                    .map_err(|e| error!("error with play-pause: {:?}", e));
                            }
                        }
                    }
                    _ => (),
                };
            }
        })
    };

    let register = async {
        write
            .send(Message::Binary(
                json!({"event": event.clone(), "uuid": uuid.clone()})
                    .to_string()
                    .into_bytes(),
            ))
            .await
            .expect("error sending registration message");

        info!("sent registration message 😃");

        write
            .send(Message::Text(
                json!({
                    "event": "logMessage",
                    "payload": {
                        "message": "AND YOU MIGHT FIND YOURSELF IN ANOTHER PART OF THE WORLD"
                    }
                })
                .to_string(),
            ))
            .await
            .expect("error logging");

        info!("logged 📝");
    };

    pin_mut!(register, read_ws, get_zone);
    futures::join!(get_zone, register, read_ws);
    info!("goodbye");

    Ok(())
}
