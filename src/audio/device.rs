use crate::audio::ProperStoreHelpers;
use windows::core::ComInterface;
use windows::Win32::Devices::FunctionDiscovery::{
    PKEY_Device_DeviceDesc, PKEY_Device_FriendlyName,
};
use windows::Win32::Media::Audio::{EDataFlow, IMMDevice, IMMEndpoint};
use windows::Win32::System::Com::STGM_READ;

#[derive(Debug, Clone)]
pub struct Device {
    pub mode: Direction,
    pub state: DeviceState,
    pub endpoint_id: String,
    pub friendly_name: Option<String>,
    pub description: Option<String>,
}

impl TryFrom<IMMDevice> for Device {
    type Error = eyre::Error;
    fn try_from(imm_device: IMMDevice) -> eyre::Result<Self> {
        unsafe {
            let endpoint_id = imm_device.GetId()?.to_string()?;
            let endpoint: IMMEndpoint = imm_device.cast()?;
            let prop_store = imm_device.OpenPropertyStore(STGM_READ)?;

            let friendly_name = prop_store.get_prop_string(PKEY_Device_FriendlyName);
            let description = prop_store.get_prop_string(PKEY_Device_DeviceDesc);

            Ok(Device {
                mode: endpoint.GetDataFlow()?.into(),
                state: imm_device.GetState()?.into(),
                endpoint_id,
                friendly_name,
                description,
            })
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Direction {
    Input,
    Output,
}

impl From<EDataFlow> for Direction {
    fn from(val: EDataFlow) -> Self {
        match val.0 {
            0 => Direction::Output,
            1 => Direction::Input,
            dir => panic!("Invalid direction: {}", dir),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum DeviceState {
    Unknown,
    Active,
    Disabled,
    NotPresent,
    Unplugged,
}

impl From<u32> for DeviceState {
    fn from(val: u32) -> Self {
        match val {
            1 => DeviceState::Active,
            2 => DeviceState::Disabled,
            4 => DeviceState::NotPresent,
            8 => DeviceState::Unplugged,
            _ => DeviceState::Unknown,
        }
    }
}
