mod audio;
mod device;

use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
use windows::Win32::{Media::Audio::*, UI::Shell::PropertiesSystem::PROPERTYKEY};

// Re-export some stuff
pub(crate) use audio::Audio;
pub(crate) use device::{Device, Direction};

const DEFAULT_STATE_MASK: u32 =
    DEVICE_STATE_ACTIVE | DEVICE_STATE_DISABLED | DEVICE_STATE_UNPLUGGED;

/// Commands the Audio thread can handle
#[derive(Debug)]
enum Request {
    #[allow(dead_code)]
    Ping,
    Shutdown,
    Devices,
    DefaultDevice(Direction),
}

/// Responses the Audio thread can reply with
#[derive(Debug)]
enum Response {
    Pong,
    Devices(Vec<Device>),
    DefaultDevice(Device),
}

/// Some helpers for working with an IPropertyStore
trait ProperStoreHelpers {
    /// Get a property as a String
    unsafe fn get_prop_string(&self, key: PROPERTYKEY) -> Option<String>;
}

impl ProperStoreHelpers for IPropertyStore {
    unsafe fn get_prop_string(&self, key: PROPERTYKEY) -> Option<String> {
        let val = self
            .GetValue(&key as *const _)
            .map_err(|e| e.to_owned())
            .ok()?;
        let val2 = val.Anonymous.Anonymous;
        // See https://learn.microsoft.com/en-us/windows/win32/api/wtypes/ne-wtypes-varenum
        match val2.vt.0 {
            31 => val2.Anonymous.pwszVal.to_string().ok(),
            _ => None,
        }
    }
}
