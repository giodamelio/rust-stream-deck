

use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc};
use std::thread::JoinHandle;
use std::{thread};

use eyre::{bail, eyre, Result, WrapErr};
use windows::core::ComInterface;
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_DeviceDesc;
use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
use windows::Win32::{
    Devices::FunctionDiscovery::{PKEY_Device_FriendlyName},
    Media::Audio::*,
    System::Com::*,
    UI::Shell::PropertiesSystem::PROPERTYKEY,
};

const DEFAULT_STATE_MASK: u32 =
    DEVICE_STATE_ACTIVE | DEVICE_STATE_DISABLED | DEVICE_STATE_UNPLUGGED;

#[derive(Debug)]
pub struct Audio {
    thread_handle: Option<JoinHandle<Result<()>>>,
    command_tx: Sender<AudioCommand>,
    response_rx: Receiver<AudioResponse>,
}

/// Commands the Audio thread can handle
#[derive(Debug)]
enum AudioCommand {
    #[allow(dead_code)]
    Ping,
    Shutdown,
    Devices,
}

/// Responses the Audio thread can reply with
#[derive(Debug)]
enum AudioResponse {
    Pong,
    Devices(Vec<Device>),
}

#[derive(Debug, Clone)]
pub struct Device {
    pub mode: Direction,
    pub state: DeviceState,
    pub endpoint_id: String,
    pub friendly_name: Option<String>,
    pub description: Option<String>,
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

impl Audio {
    pub fn new() -> Self {
        let (command_tx, command_rx) = mpsc::channel();
        let (response_tx, response_rx) = mpsc::channel();

        let thread_handle = thread::spawn(move || -> Result<()> {
            unsafe {
                // Initialize things
                CoInitializeEx(None, COINIT_MULTITHREADED)?;

                // Create the device enumerator
                let enumerator: IMMDeviceEnumerator =
                    CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

                for command in command_rx.iter() {
                    match command {
                        AudioCommand::Ping => {
                            println!("PING");
                            response_tx.send(AudioResponse::Pong)?
                        }
                        AudioCommand::Shutdown => {
                            break;
                        }
                        AudioCommand::Devices => {
                            let mut devices: Vec<Device> = vec![];

                            unsafe fn get_prop_string(
                                prop_store: &IPropertyStore,
                                key: PROPERTYKEY,
                            ) -> Option<String> {
                                let val = prop_store
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

                            // Get all audio devices
                            let device_collection =
                                enumerator.EnumAudioEndpoints(eAll, DEFAULT_STATE_MASK)?;
                            for n in 0..device_collection.GetCount()? {
                                let device = device_collection.Item(n)?;
                                let endpoint_id = device.GetId()?.to_string()?;
                                let endpoint: IMMEndpoint = device.cast()?;
                                let prop_store = device.OpenPropertyStore(STGM_READ)?;

                                let friendly_name =
                                    get_prop_string(&prop_store, PKEY_Device_FriendlyName);
                                let description =
                                    get_prop_string(&prop_store, PKEY_Device_DeviceDesc);

                                devices.push(Device {
                                    mode: endpoint.GetDataFlow()?.into(),
                                    state: device.GetState()?.into(),
                                    endpoint_id,
                                    friendly_name,
                                    description,
                                })
                            }

                            response_tx.send(AudioResponse::Devices(devices))?
                        }
                    }
                }

                Ok(())
            }
        });

        Self {
            thread_handle: Some(thread_handle),
            command_tx,
            response_rx,
        }
    }

    /// Wait for the audio thread to finish
    pub fn wait(&mut self) -> Result<()> {
        match self.thread_handle.take() {
            None => bail!("Problem waiting for Audio thread"),
            Some(handle) => handle.join().map_err(|e| eyre!("{:?}", e))?,
        }
    }

    /// Shutdown the audio thread gracefully
    pub fn shutdown(&self) -> Result<()> {
        self.command_tx
            .send(AudioCommand::Shutdown)
            .wrap_err("Problem shutting down")
    }

    /// Get a vector of the systems audio devices
    pub fn devices(&self) -> Result<Vec<Device>> {
        if let AudioResponse::Devices(devices) = self.command(AudioCommand::Devices)? {
            Ok(devices)
        } else {
            bail!("Bad response")
        }
    }

    /// Small helper to synchronously send a command to the Audio thread and wait for a response
    fn command(&self, command: AudioCommand) -> Result<AudioResponse> {
        self.command_tx.send(command)?;
        self.response_rx
            .recv()
            .wrap_err("Problem receiving response")
    }
}
