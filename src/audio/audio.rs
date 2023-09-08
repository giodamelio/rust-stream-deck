use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

use eyre::{bail, eyre, Result, WrapErr};
use windows::Win32::Media::Audio::{
    eAll, eCapture, eMultimedia, eRender, EDataFlow, IMMDeviceEnumerator, MMDeviceEnumerator,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
};

use crate::audio::{Device, Direction, Request, Response, DEFAULT_STATE_MASK};

#[derive(Debug)]
pub struct Audio {
    thread_handle: Option<JoinHandle<Result<()>>>,
    command_tx: Sender<Request>,
    response_rx: Receiver<Response>,
}

impl From<Direction> for EDataFlow {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Input => eCapture,
            Direction::Output => eRender,
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
                        Request::Ping => {
                            println!("PING");
                            response_tx.send(Response::Pong)?
                        }
                        Request::Shutdown => {
                            break;
                        }
                        Request::DefaultDevice(direction) => {
                            let default_device = enumerator
                                .GetDefaultAudioEndpoint(direction.into(), eMultimedia)?;
                            response_tx.send(Response::DefaultDevice(default_device.try_into()?))?
                        }
                        Request::Devices => {
                            let mut devices: Vec<Device> = vec![];

                            // Get all audio devices
                            let device_collection =
                                enumerator.EnumAudioEndpoints(eAll, DEFAULT_STATE_MASK)?;
                            for n in 0..device_collection.GetCount()? {
                                let device = device_collection.Item(n)?;
                                devices.push(device.try_into()?);
                            }

                            response_tx.send(Response::Devices(devices))?
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
        if let Response::Devices(devices) = self.command(Request::Devices)? {
            Ok(devices)
        } else {
            bail!("Bad response")
        }
    }

    /// Get the default output or input device
    pub fn default_device(&self, direction: Direction) -> Result<Device> {
        if let Response::DefaultDevice(device) = self.command(Request::DefaultDevice(direction))? {
            Ok(device)
        } else {
            bail!("Bad response")
        }
    }

    /// Small helper to synchronously send a command to the Audio thread and wait for a response
    fn command(&self, command: Request) -> Result<Response> {
        self.command_tx.send(command)?;
        self.response_rx
            .recv()
            .wrap_err("Problem receiving response")
    }
}
