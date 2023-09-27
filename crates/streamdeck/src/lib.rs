mod device;
pub mod device_constants;
mod error;
mod model;

use crate::device::DeviceInfo;

// Re-export things
pub use device::Device;
pub use model::Model;

pub(crate) fn list_devices() -> Vec<DeviceInfo> {
    let api = hidapi::HidApi::new().unwrap();
    api.device_list()
        .filter_map(|info| info.try_into().ok())
        .collect::<Vec<DeviceInfo>>()
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

// These tests fail if at least one streamdeck device is not connected
#[cfg(test)]
mod tests {
    use hidapi::HidApi;

    use super::*;

    // Helper to get the first device
    fn get_first_device() -> Device {
        let api = HidApi::new().unwrap();
        let devices = list_devices();
        let info = devices.first().unwrap();
        Device::from_device_info(&api, info).unwrap()
    }

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn should_list_a_device() {
        let devices = list_devices();
        assert_eq!(devices.len(), 1);
    }

    #[test]
    fn get_serial_number() {
        let device = get_first_device();
        assert!(device.serial_number().unwrap().is_some());
    }
}
