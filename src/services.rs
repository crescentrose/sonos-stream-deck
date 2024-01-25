use std::fmt::Display;

use crate::error::ControllerError;
use rupnp::{ssdp::URN, Device, Service};

#[derive(Debug, Clone)]
pub struct AVTransport {
    service: Service,
}

impl AVTransport {
    const SERVICE_URN: URN = URN::service("schemas-upnp-org", "AVTransport", 1);

    pub fn from_device(device: &Device) -> Result<Self, ControllerError> {
        let service =
            device
                .find_service(&Self::SERVICE_URN)
                .ok_or(ControllerError::ServiceUnavailable(
                    "AVTransport".to_string(),
                    device.friendly_name().to_string(),
                ))?;

        Ok(Self {
            service: service.clone(),
        })
    }

    pub async fn pause(&self, device: &Device) -> Result<(), ControllerError> {
        let payload = "<InstanceID>0</InstanceID>";
        self.service.action(device.url(), "Pause", payload).await?;
        Ok(())
    }

    pub async fn play(&self, device: &Device) -> Result<(), ControllerError> {
        let payload = "<InstanceID>0</InstanceID><Speed>1</Speed>";
        self.service.action(device.url(), "Play", payload).await?;
        Ok(())
    }

    pub async fn previous(&self, device: &Device) -> Result<(), ControllerError> {
        let payload = "<InstanceID>0</InstanceID>";
        self.service
            .action(device.url(), "Previous", payload)
            .await?;
        Ok(())
    }

    pub async fn next(&self, device: &Device) -> Result<(), ControllerError> {
        let payload = "<InstanceID>0</InstanceID>";
        self.service.action(device.url(), "Next", payload).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RenderingControl {
    service: Service,
}

impl RenderingControl {
    const SERVICE_URN: URN = URN::service("schemas-upnp-org", "RenderingControl", 1);

    pub fn from_device(device: &Device) -> Result<Self, ControllerError> {
        let service =
            device
                .find_service(&Self::SERVICE_URN)
                .ok_or(ControllerError::ServiceUnavailable(
                    "RenderingControl".to_string(),
                    device.friendly_name().to_string(),
                ))?;

        Ok(Self {
            service: service.clone(),
        })
    }

    pub async fn get_volume(&self, device: &Device) -> Result<Volume, ControllerError> {
        let payload = "<InstanceID>0</InstanceID><Channel>Master</Channel>";
        let resp = self
            .service
            .action(device.url(), "GetVolume", payload)
            .await?;
        let volume = resp.get("CurrentVolume");
        Ok(Volume::try_from(volume)?)
    }

    pub async fn set_volume(
        &self,
        device: &Device,
        volume: &Volume,
    ) -> Result<(), ControllerError> {
        let payload = format!(
            "<InstanceID>0</InstanceID><Channel>Master</Channel><DesiredVolume>{}</DesiredVolume>",
            &volume.value()
        );

        self.service
            .action(device.url(), "GetVolume", &payload)
            .await?;
        Ok(())
    }
}

pub struct Volume(u8);

impl Volume {
    const MAX_VOLUME: u8 = 100;

    pub fn new(value: u8) -> Self {
        if value > Self::MAX_VOLUME {
            Self(Self::MAX_VOLUME)
        } else {
            Self(value)
        }
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

impl TryFrom<Option<&String>> for Volume {
    type Error = ControllerError;

    fn try_from(value: Option<&String>) -> Result<Self, ControllerError> {
        let value = value.ok_or(ControllerError::VolumeError)?;
        Ok(Self::new(
            value.parse().map_err(|_| ControllerError::VolumeError)?,
        ))
    }
}
impl Display for Volume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.value()))
    }
}
