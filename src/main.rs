mod audio;

use clap::{Parser, Subcommand};
use eyre::Result;
use std::process;

use crate::audio::Device;
use audio::{Audio, Direction};

/// Alternate StreamDeck companion software
#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Help troubleshoot system
    #[command(arg_required_else_help = true)]
    Debug {
        #[clap(subcommand)]
        command: DebugCommand,
    },
}

#[derive(Debug, Subcommand)]
enum DebugCommand {
    /// Audio related tools
    Audio {
        #[clap(subcommand)]
        command: AudioCommand,
    },
}

#[derive(Debug, Subcommand)]
enum AudioCommand {
    /// List audio devices
    ListDevices,
    /// List audio streams for a device
    ListStreams { device: String },
}

fn wrapped() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => {
            todo!("Start the main application");
        }
        Some(command) => match command {
            Command::Debug { command } => match command {
                DebugCommand::Audio { command } => match command {
                    AudioCommand::ListDevices => {
                        let mut audio = Audio::new();

                        // Get a sorted list of devices
                        let mut devices = audio.devices()?;
                        devices.sort_by_key(|d| match d.friendly_name.clone() {
                            Some(name) => name,
                            None => "<no_friendly_name>".to_string(),
                        });

                        /// Print devices in a pretty tree format
                        fn print_device(device: &Device, is_last: bool) {
                            let gap = if is_last { "    " } else { "│   " };
                            println!(
                                "{}{}",
                                if is_last { "└── " } else { "├── " },
                                device
                                    .friendly_name
                                    .clone()
                                    .unwrap_or("<No friendly name>".to_owned())
                            );
                            println!("{}├── State:        {:?}", gap, device.state);
                            println!("{}├── Mode:         {:?}", gap, device.mode);
                            println!(
                                "{}├── Description:  {}",
                                gap,
                                device
                                    .description
                                    .clone()
                                    .unwrap_or("<no description>".to_string())
                            );
                            println!("{}└── Endpoint ID:  {}", gap, device.endpoint_id.clone());
                        }

                        fn print_devices(title: &'static str, devices: Vec<&Device>) {
                            println!("{}", title);
                            let mut iterator = devices.iter().peekable();
                            while let Some(device) = iterator.next() {
                                let is_last = iterator.peek().is_none();
                                print_device(device, is_last);
                            }
                        }

                        // Split input and output devices
                        let (input_devices, output_devices): (Vec<_>, Vec<_>) =
                            devices.iter().partition(|d| d.mode == Direction::Input);

                        // Print the devices
                        print_devices("Input devices", input_devices);
                        print_devices("Output devices", output_devices);

                        audio.shutdown()?;
                        audio.wait()?;

                        Ok(())
                    }
                    AudioCommand::ListStreams { device } => {
                        todo!("Listing streams for device={}", device);
                    }
                },
            },
        },
    }
}

fn main() {
    match wrapped() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    }
}
