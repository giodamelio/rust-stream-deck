use elgato_streamdeck::StreamDeck;
use eyre::Result;
use hidapi::HidApi;

pub fn run() -> Result<()> {
    let hid = HidApi::new()?;
    let devices = elgato_streamdeck::list_devices(&hid);

    println!("{} StreamDeck devices are connected\n", devices.len());

    // Connect to each device to get it's info
    for (kind, serial) in devices {
        let device = StreamDeck::connect(&hid, kind, &serial)?;

        println!("Device Model: {}", device.product()?);
        println!("├── Kind: {:?}", device.kind());
        println!("├── Serial Number: {}", device.serial_number()?);
        println!("└── Firmware Version: {}", device.firmware_version()?);
    }

    Ok(())
}
