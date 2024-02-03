use crate::{
    error::ControllerError,
    services::{AVTransport, AVTransportState, RenderingControl, Volume},
};
use futures::{pin_mut, prelude::*};
use rupnp::{
    ssdp::{SearchTarget, URN},
    Device,
};
use std::time::Duration;

const ZONE_GROUP_TOPOLOGY: URN = URN::service("schemas-upnp-org", "ZoneGroupTopology", 1);

#[derive(Clone, Debug)]
pub struct Zone {
    primary_device: Device,
    av_transport: AVTransport,
    rendering_control: RenderingControl,
}

impl Zone {
    pub async fn get_zones(timeout: Duration) -> Result<Vec<Zone>, ControllerError> {
        let search_target = SearchTarget::URN(ZONE_GROUP_TOPOLOGY);
        let devices = rupnp::discover(&search_target, timeout).await?;

        let mut zones: Vec<Zone> = vec![];

        pin_mut!(devices);
        while let Some(device) = devices.try_next().await? {
            let service = device
                .find_service(&ZONE_GROUP_TOPOLOGY)
                .expect("searched for ZoneGroupTopology, got something else");

            let args = "";
            let response = service
                .action(device.url(), "GetZoneGroupAttributes", args)
                .await?;

            if response.get("CurrentZoneGroupID").is_none() {
                continue;
            }

            let zone = Zone::from_device(device);
            zones.push(zone);
        }

        Ok(zones)
    }

    pub fn from_device(primary_device: Device) -> Self {
        let av_transport =
            AVTransport::from_device(&primary_device).expect("expected AVTransport on device");
        let rendering_control = RenderingControl::from_device(&primary_device)
            .expect("expected RenderingControl on device");
        Zone {
            primary_device,
            av_transport,
            rendering_control,
        }
    }

    pub async fn pause(&self) -> Result<(), ControllerError> {
        self.av_transport.pause(&self.primary_device).await
    }

    pub async fn play(&self) -> Result<(), ControllerError> {
        self.av_transport.play(&self.primary_device).await
    }

    pub async fn next(&self) -> Result<(), ControllerError> {
        self.av_transport.next(&self.primary_device).await
    }

    pub async fn previous(&self) -> Result<(), ControllerError> {
        self.av_transport.previous(&self.primary_device).await
    }

    pub async fn play_pause(&self) -> Result<(), ControllerError> {
        match self.get_state().await? {
            AVTransportState::Paused | AVTransportState::Stopped => self.play().await,
            AVTransportState::Playing | AVTransportState::Transitioning => self.pause().await,
        }
    }

    pub async fn get_state(&self) -> Result<AVTransportState, ControllerError> {
        self.av_transport
            .get_transport_info(&self.primary_device)
            .await
    }

    pub async fn get_volume(&self) -> Result<Volume, ControllerError> {
        self.rendering_control
            .get_volume(&self.primary_device)
            .await
    }

    pub async fn set_volume(&self, volume: &Volume) -> Result<(), ControllerError> {
        self.rendering_control
            .set_volume(&self.primary_device, &volume)
            .await
    }

    pub fn name(&self) -> &str {
        self.primary_device.friendly_name()
    }
}
