use crate::deck::Deck;
use image::RgbImage;

pub fn run() -> eyre::Result<()> {
    // Make an all white image
    let mut img = RgbImage::new(72, 72);
    img.fill(255);

    // Get the first device
    let device = Deck::list_devices()?.remove(0);

    // Set every button to the white image
    for button_index in 0..device.kind.key_count() {
        device.set_button_image(button_index, img.clone().into())?;
    }

    // If the device has a screen, create an image for it and set it
    if let Some((width, height)) = device.kind.lcd_strip_size() {
        let mut img = RgbImage::new(width as u32, height as u32);
        img.fill(255);
        device.set_lcd_image(img.into())?;
    }

    Ok(())
}
