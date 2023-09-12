use elgato_streamdeck::images::ImageRect;
use std::fmt::{Debug, Formatter};

use elgato_streamdeck::info::Kind;
use elgato_streamdeck::StreamDeck;
use eyre::{Result, WrapErr};
use hidapi::HidApi;
use image::DynamicImage;

pub struct Deck {
    pub kind: Kind,
    pub product: String,
    pub serial_number: String,
    pub firmware_version: String,

    device: StreamDeck,
}

impl Debug for Deck {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (serial_number={}, firmware_version={})",
            self.product, self.serial_number, self.firmware_version
        )
    }
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
                device,
            });
        }

        Ok(decks)
    }

    pub fn set_button_image(&self, index: u8, image: DynamicImage) -> Result<()> {
        self.device
            .set_button_image(index, image)
            .wrap_err("Problem setting button image")
    }

    pub fn set_lcd_image(&self, image: DynamicImage) -> Result<()> {
        self.device
            .write_lcd(0, 0, &ImageRect::from_image(image)?)
            .wrap_err("Problem setting lcd image")
    }
}
