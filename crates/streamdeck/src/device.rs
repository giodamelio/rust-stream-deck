use std::fmt::{Debug, Display, Formatter};
use std::io::{self, Cursor, ErrorKind, Read, Write};

use binrw::prelude::*;
use hidapi::{HidApi, HidDevice, HidError};

use crate::device_constants::*;
use crate::error::{StreamDeckError, StreamDeckResult};
use crate::model::Model;

pub struct DeviceInfo(hidapi::DeviceInfo, Model);

impl DeviceInfo {
    pub fn model(&self) -> &Model {
        &self.1
    }
}

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
        let device = self.get_device().ok()?;
        let info = device.get_device_info().ok()?;
        let c_path = info.path();
        let path_ref = c_path.to_str().ok()?;
        Some(path_ref.to_string())
    }

    pub fn serial_number(&self) -> StreamDeckResult<String> {
        let report = self.get_feature_report::<32>(6)?;
        let feature: FeatureReportSerialNumber = Cursor::new(report).read_be()?;
        Ok(feature.serial.to_string())
    }

    pub fn product_string(&self) -> StreamDeckResult<Option<String>> {
        let device = self.get_device()?;
        Ok(device.get_product_string()?)
    }

    pub fn firmware_version(&self) -> StreamDeckResult<String> {
        let report = self.get_feature_report::<32>(5)?;
        let feature: FeatureReportFirmwareVersion = Cursor::new(report).read_be()?;
        Ok(feature.version.to_string())
    }

    // Helper to get the device
    fn get_device(&self) -> StreamDeckResult<&HidDevice> {
        self.device
            .as_ref()
            .ok_or(StreamDeckError::DeviceNotConnected)
    }

    pub fn get_feature_report<const S: usize>(&self, report_id: u8) -> StreamDeckResult<Vec<u8>> {
        let device = self.get_device()?;
        let mut buf = [0u8; S];
        buf[0] = report_id;
        let len = device.get_feature_report(&mut buf)?;
        Ok(buf.iter().cloned().take(len + 1).collect())
    }
}

impl Read for Device {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let device = self
            .get_device()
            .map_err(|err| io::Error::new(ErrorKind::Other, err))?;

        let result = device.read(buf).map_err(|err| match err {
            HidError::IoError { error } => error,
            err => io::Error::new(ErrorKind::Other, err),
        });

        result
    }
}

impl Write for Device {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let device = self
            .get_device()
            .map_err(|err| io::Error::new(ErrorKind::Other, err))?;

        device.write(buf).map_err(|err| match err {
            HidError::IoError { error } => error,
            err => io::Error::new(ErrorKind::Other, err),
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
