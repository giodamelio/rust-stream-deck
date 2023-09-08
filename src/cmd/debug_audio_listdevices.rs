use eyre::Result;

use crate::audio::{Audio, Device, Direction};
pub fn run() -> Result<()> {
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
