mod error;
mod services;
mod zone;

use error::ControllerError;
use std::time::Duration;
use zone::Zone;

use crate::services::Volume;

#[tokio::main]
async fn main() -> Result<(), ControllerError> {
    demo().await
}

async fn demo() -> Result<(), ControllerError> {
    let a_bit = Duration::from_secs(4);

    // discovery
    let zones = Zone::get_zones(Duration::from_secs(3)).await?;

    // control
    for zone in zones.iter() {
        println!("controling {}", zone.name());
        println!("volume: {}", zone.get_volume().await?);

        println!("set volume to 30");
        zone.set_volume(&Volume::new(30)).await?;

        println!("play");
        zone.play().await?;
        tokio::time::sleep(a_bit).await;

        println!("next");
        zone.next().await?;
        tokio::time::sleep(a_bit).await;

        println!("set volume to 20");
        zone.set_volume(&Volume::new(20)).await?;
        println!("previous");
        zone.previous().await?;
        tokio::time::sleep(a_bit).await;

        println!("pause");
        zone.pause().await?;
    }

    Ok(())
}
