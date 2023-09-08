mod device;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

use eyre::{bail, eyre, Result, WrapErr};
use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
use windows::Win32::{Media::Audio::*, System::Com::*, UI::Shell::PropertiesSystem::PROPERTYKEY};

// Re-export some stuff
pub(crate) use device::{Device, Direction};

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
    DefaultDevice(Direction),
}

/// Responses the Audio thread can reply with
#[derive(Debug)]
enum AudioResponse {
    Pong,
    Devices(Vec<Device>),
    DefaultDevice(Device),
}

impl From<Direction> for EDataFlow {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Input => eCapture,
            Direction::Output => eRender,
        }
    }
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
                        AudioCommand::DefaultDevice(direction) => {
                            let default_device = enumerator
                                .GetDefaultAudioEndpoint(direction.into(), eMultimedia)?;
                            response_tx
                                .send(AudioResponse::DefaultDevice(default_device.try_into()?))?
                        }
                        AudioCommand::Devices => {
                            let mut devices: Vec<Device> = vec![];

                            // Get all audio devices
                            let device_collection =
                                enumerator.EnumAudioEndpoints(eAll, DEFAULT_STATE_MASK)?;
                            for n in 0..device_collection.GetCount()? {
                                let device = device_collection.Item(n)?;
                                devices.push(device.try_into()?);
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

    /// Get the default output or input device
    pub fn default_device(&self, direction: Direction) -> Result<Device> {
        if let AudioResponse::DefaultDevice(device) =
            self.command(AudioCommand::DefaultDevice(direction))?
        {
            Ok(device)
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
