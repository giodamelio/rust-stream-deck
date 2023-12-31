mod audio;
mod cmd;

use clap::{Args, Parser, Subcommand};
use eyre::Result;
use std::process;

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
    ListStreams(ListStreamsOptions),
}

#[derive(Debug, Args)]
pub struct ListStreamsOptions {
    /// The device to list streams for. If no device is specified, the default output will be used
    device: Option<String>,
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
                    AudioCommand::ListDevices => cmd::debug_audio_listdevices(),
                    AudioCommand::ListStreams(options) => cmd::debug_audio_liststreams(options),
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
