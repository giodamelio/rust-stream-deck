use hidapi::HidError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StreamDeckError {
    #[error("vendor id `{0}` does not match elgato")]
    InvalidVendorID(u16),
    #[error("product id `{0}` does not match any streamdeck device")]
    InvalidProductID(u16),
    #[error("device is not connected")]
    DeviceNotConnected,
    #[error("hid error: {0}")]
    HidError(#[from] HidError),
}

pub type StreamDeckResult<T> = Result<T, StreamDeckError>;
