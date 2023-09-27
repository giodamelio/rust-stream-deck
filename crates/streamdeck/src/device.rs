use std::fmt::{Debug, Display, Formatter};

use crate::error::{StreamDeckError, StreamDeckResult};
use hidapi::{HidApi, HidDevice};

use crate::model::Model;

pub struct DeviceInfo(hidapi::DeviceInfo, Model);

impl TryFrom<&hidapi::DeviceInfo> for DeviceInfo {
    type Error = StreamDeckError;

    fn try_from(info: &hidapi::DeviceInfo) -> Result<Self, Self::Error> {
        let model = Model::from_device_info(&info)?;
        Ok(Self(info.clone(), model))
    }
}

impl Debug for DeviceInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.1, f)
    }
}

pub struct Device {
    model: Model,
    device: Option<HidDevice>,
}

impl Debug for Device {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display_message = self.to_string();

        if !self.connected() {
            return write!(f, "{} (Disconnected)", display_message);
        }

        let path = match self.path() {
            Some(path) => path,
            None => "unknown".to_string(),
        };

        write!(f, "{} (Connected as {})", display_message, path)
    }
}

impl Display for Device {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if !self.connected() {
            return write!(f, "Unknown StreamDeck");
        }

        write!(f, "StreamDeck {}", self.model())
    }
}

impl Device {
    pub fn from_device_info(
        hid_api: &HidApi,
        device_info: &DeviceInfo,
    ) -> StreamDeckResult<Device> {
        // TODO: should we connect to path in case there are multiple devices
        let device = device_info.0.open_device(hid_api)?;

        Ok(Device {
            model: device_info.1.clone(),
            device: Some(device),
        })
    }

    pub fn from_hid_device_info(
        hid_api: &HidApi,
        device_info: &hidapi::DeviceInfo,
    ) -> StreamDeckResult<Device> {
        let model = Model::from_device_info(device_info)?;
        // TODO: should we connect to path in case there are multiple devices
        let device = device_info.open_device(hid_api)?;

        Ok(Device {
            model,
            device: Some(device),
        })
    }

    pub fn connected(&self) -> bool {
        self.device.is_some()
    }

    pub fn model(&self) -> String {
        self.model.to_string()
    }

    pub fn path(&self) -> Option<String> {
        let device = self.get_device()?;
        let info = device.get_device_info().ok()?;
        let c_path = info.path();
        let path_ref = c_path.to_str().ok()?;
        Some(path_ref.to_string())
    }

    pub fn serial_number(&self) -> StreamDeckResult<Option<String>> {
        let device = self
            .get_device()
            .ok_or(StreamDeckError::DeviceNotConnected)?;
        Ok(device.get_serial_number_string()?)
    }

    fn get_device(&self) -> Option<&HidDevice> {
        self.device.as_ref()
    }
}
