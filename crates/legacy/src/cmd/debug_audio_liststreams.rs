use eyre::Result;

use crate::audio::{Audio, Direction};
use crate::ListStreamsOptions;

pub fn run(options: ListStreamsOptions) -> Result<()> {
    let mut audio = Audio::new();
    let device = match options.device {
        None => audio.default_device(Direction::Output)?,
        Some(_device_name) => todo!("Get device by name or id or something"),
    };

    println!(
        "Listing audio streams for device: {}",
        device
            .clone()
            .friendly_name
            .unwrap_or(String::from("<no friendly name>"))
    );
    println!();

    for stream in device.streams()? {
        println!(
            "{}",
            stream
                .friendly_name
                .unwrap_or(String::from("<no friendly_name>"))
        );
        println!("├── State:      {:?}", stream.state);
        println!("└── Process ID: {}", stream.process_id);
    }

    audio.shutdown()?;
    audio.wait()?;

    Ok(())
}
