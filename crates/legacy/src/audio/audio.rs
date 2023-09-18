use std::thread::JoinHandle;
use std::{ptr, thread};

use crossbeam::channel;
use eyre::{bail, eyre, Result};
use windows::core::ComInterface;
use windows::Win32::Media::Audio::{
    eAll, eCapture, eMultimedia, eRender, EDataFlow, IAudioSessionControl2, IAudioSessionManager2,
    IMMDeviceEnumerator, MMDeviceEnumerator,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
};

use crate::audio::{
    CommandChannel, Commandable, Device, Direction, Request, Response, Stream, DEFAULT_STATE_MASK,
};

#[derive(Debug)]
pub struct Audio {
    thread_handle: Option<JoinHandle<Result<()>>>,
    command_channel: CommandChannel,
}

impl Commandable for Audio {
    fn command(&self, command: Request) -> Result<Response> {
        self.command_channel.command(command)
    }
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
        let (command_tx, command_rx) = channel::unbounded();
        let (response_tx, response_rx) = channel::unbounded();
        let command_channel = CommandChannel::new(command_tx, response_rx);
        let command_channel_for_audio = command_channel.clone();

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
                            response_tx.send(Response::DefaultDevice(
                                (default_device, command_channel_for_audio.clone()).try_into()?,
                            ))?
                        }
                        Request::Devices => {
                            let mut devices: Vec<Device> = vec![];

                            // Get all audio devices
                            let device_collection =
                                enumerator.EnumAudioEndpoints(eAll, DEFAULT_STATE_MASK)?;
                            for n in 0..device_collection.GetCount()? {
                                let device = device_collection.Item(n)?;
                                devices
                                    .push((device, command_channel_for_audio.clone()).try_into()?);
                            }

                            response_tx.send(Response::Devices(devices))?
                        }
                        Request::Streams(device) => {
                            let mut streams: Vec<Stream> = vec![];

                            let raw_device = enumerator.GetDevice(&device.endpoint_id)?;
                            let manager: IAudioSessionManager2 =
                                raw_device.Activate(CLSCTX_ALL, Some(ptr::null()))?;
                            let session_collection = manager.GetSessionEnumerator()?;
                            for n in 0..session_collection.GetCount()? {
                                let session = session_collection.GetSession(n)?;
                                // I am honestly not sure why I can't just unwrap normally
                                // the windows::core::Result part is somehow very important
                                let session2 = {
                                    let s: windows::core::Result<IAudioSessionControl2> =
                                        session.cast();
                                    s?
                                };

                                let friendly_name = match session.GetDisplayName()?.to_string()? {
                                    name if name.is_empty() => None,
                                    name => Some(name),
                                };
                                let process_id = session2.GetProcessId().unwrap();

                                streams.push(Stream {
                                    friendly_name,
                                    state: session.GetState()?.into(),
                                    process_id,
                                });
                            }

                            response_tx.send(Response::Streams(streams))?
                        }
                    }
                }

                Ok(())
            }
        });

        Self {
            thread_handle: Some(thread_handle),
            command_channel,
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
        let CommandChannel(command_tx, _response_rx) = &self.command_channel;
        command_tx.send(Request::Shutdown).map_err(|e| eyre!(e))
        // .wrap_err("Problem shutting down")
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
}
