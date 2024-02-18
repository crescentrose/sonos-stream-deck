use thiserror::Error;

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error("service {0} not available on device {1}")]
    ServiceUnavailable(String, String),
    #[error("transport error")]
    TransportError(#[from] rupnp::Error),
    #[error("volume should be an integer between 0 and 100")]
    VolumeError,
    #[error("response malformed")]
    MalformedResponse,
}
