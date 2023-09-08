use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

use anyhow::{anyhow, bail, Context, Result};

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

#[derive(Debug, Copy, Clone)]
pub struct Device {
    pub mode: Direction,
}

#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Input,
    Output,
}

impl Audio {
    pub fn new() -> Self {
        let (command_tx, command_rx) = mpsc::channel();
        let (response_tx, response_rx) = mpsc::channel();

        let thread_handle = thread::spawn(move || -> Result<()> {
            let devices = vec![
                Device {
                    mode: Direction::Output,
                },
                Device {
                    mode: Direction::Input,
                },
            ];

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
                        response_tx.send(AudioResponse::Devices(devices.clone()))?
                    }
                }
            }

            Ok(())
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
            Some(handle) => handle.join().map_err(|e| anyhow!("{:?}", e))?,
        }
    }

    /// Shutdown the audio thread gracefully
    pub fn shutdown(&self) -> Result<()> {
        self.command_tx
            .send(AudioCommand::Shutdown)
            .context("Problem shutting down")
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
            .context("Problem receiving response")
    }
}
