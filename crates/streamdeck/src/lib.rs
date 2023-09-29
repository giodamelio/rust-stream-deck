mod device;
pub mod device_constants;
mod error;
mod model;

use hidapi::HidApi;

use crate::device::DeviceInfo;

// Re-export things
pub use device::Device;
pub use model::Model;

pub fn list_devices() -> Vec<DeviceInfo> {
    let api = hidapi::HidApi::new().unwrap();
    api.device_list()
        .filter_map(|info| info.try_into().ok())
        .collect::<Vec<DeviceInfo>>()
}

#[doc(hidden)]
/// Helper method mostly used for testing
pub fn get_first_device() -> Device {
    let api = HidApi::new().unwrap();
    let devices = list_devices();
    let info = devices.first().unwrap();
    Device::from_device_info(&api, info).unwrap()
}

// These tests are all based on my StreamDeck Plus, they will fail if no StreamDeck or another model StreamDeck is connected
#[cfg(test)]
mod tests {
    use std::io;
    use std::io::Read;

    use super::*;

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

    #[test]
    fn get_product_string() {
        let device = get_first_device();
        assert_eq!(
            device.product_string().unwrap().unwrap(),
            "Stream Deck Plus"
        );
    }

    #[test]
    fn read_some_bytes() -> io::Result<()> {
        let mut device = get_first_device();

        println!("About to start reading bytes");
        for byte in device.bytes() {
            println!("{}", byte.unwrap());
        }

        assert!(false);
        Ok(())
    }
}
