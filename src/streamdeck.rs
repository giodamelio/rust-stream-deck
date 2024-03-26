use anyhow::{anyhow, Result};
use bevy::app::{Plugin, PreStartup};
use elgato_streamdeck::StreamDeck;

pub struct StreamDeckPlugin;

impl Plugin for StreamDeckPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PreStartup, listener);
    }
}

fn listener() {
    let _deck = get_exactly_one_streamdeck();
}

// Get the StreamDeck device
// Error if there is more then one available
fn get_exactly_one_streamdeck() -> Result<StreamDeck> {
    let hid = elgato_streamdeck::new_hidapi()?;
    let devices = elgato_streamdeck::list_devices(&hid);

    let (kind, serial) = devices
        .first()
        .ok_or(anyhow!("Only exactly one StreamDeck at a time"))?;

    let device = StreamDeck::connect(&hid, *kind, serial)?;

    Ok(device)
}

// +use elgato_streamdeck::StreamDeck;
// +
// +fn main() -> Result<(), Box<dyn std::error::Error>> {
// +    // Create instance of HidApi
// +    let hid = elgato_streamdeck::new_hidapi()?;
// +
// +    // List devices and unsafely take first one
// +    let devices = elgato_streamdeck::list_devices(&hid);
// +    let (kind, serial) = devices.first().ok_or("OH NO")?;
// +    println!("{:?}, {:?}", kind, serial);
// +
// +    // Connect to the device
// +    let device = StreamDeck::connect(&hid, *kind, &serial).expect("Failed to connect");
// +
// +    // Print out some info from the device
// +    println!(
// +        "Connected to '{}' with version '{}'",
// +        device.serial_number()?,
// +        device.firmware_version()?
// +    );
// +
// +    // Set device brightness
// +    device.set_brightness(100).unwrap();
// +
// +    Ok(())
//  }
