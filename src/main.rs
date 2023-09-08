use clap::{Parser, Subcommand};

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
        command: AudioCommand
    },
}

#[derive(Debug, Subcommand)]
enum AudioCommand {
    /// List audio devices
    ListDevices,
    /// List audio streams for a device
    ListStreams {
        device: String
    },
}

fn main() {
    let cli = Cli::parse();
    println!("{:#?}", cli);
    match cli.command {
        None => {
            todo!("Start the main application");
        }
        Some(command) => {
            match command {
                Command::Debug { command } => match command {
                    DebugCommand::Audio { command } => match command {
                        AudioCommand::ListDevices => {
                            todo!("List devices");
                        }
                        AudioCommand::ListStreams { device } => {
                            todo!("Listing streams for device={}", device);
                        }
                    }
                }
            }
        }
    }

}