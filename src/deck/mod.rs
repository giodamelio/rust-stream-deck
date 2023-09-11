use elgato_streamdeck::info::Kind;
use elgato_streamdeck::StreamDeck;
use hidapi::HidApi;

pub struct Deck {
    pub kind: Kind,
    pub product: String,
    pub serial_number: String,
    pub firmware_version: String,
}

impl Deck {
    pub fn list_devices() -> eyre::Result<Vec<Deck>> {
        // Get a list of connected devices
        let mut hid = HidApi::new()?;
        elgato_streamdeck::refresh_device_list(&mut hid)?;
        let devices: Vec<(Kind, String)> = elgato_streamdeck::list_devices(&hid);

        // Connect to each of them, and extract a few bits of data
        let mut decks: Vec<Deck> = vec![];
        for (kind, serial) in devices {
            let device = StreamDeck::connect(&hid, kind, &serial)?;

            decks.push(Deck {
                kind,
                product: device.product()?,
                serial_number: device.serial_number()?,
                firmware_version: device.firmware_version()?,
            });
        }

        Ok(decks)
    }
}
