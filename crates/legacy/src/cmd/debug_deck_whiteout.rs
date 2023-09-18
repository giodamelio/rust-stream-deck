use crate::deck::{Deck, DeckRequest};
use crate::util::RequestableThread;

pub fn run() -> eyre::Result<()> {
    // Get the first device
    let mut device = Deck::list_devices()?.remove(0);
    device.start()?;

    device.set_brightness(50)?;

    dbg!(device.ping()?);

    // device.wait()?;

    // let join_handle = device.start()?;
    //
    // dbg!(device.request(DeckRequest::Ping)?);
    // device.exit()?;

    // join_handle.join().unwrap();

    // device.connect()?;

    // write_grey_to_all_screens(&device, 100)?;

    // let listener = device.add_listener();
    //
    // let mut color: u8 = 0;
    // for input in listener.iter() {
    //     if let StreamDeckInput::EncoderTwist(input) = input {
    //         let change = *input.first().unwrap();
    //         if change.is_positive() {
    //             color = match color.checked_add(change.unsigned_abs()) {
    //                 None => color,
    //                 Some(c) => c,
    //             };
    //         } else {
    //             color = match color.checked_sub(change.unsigned_abs()) {
    //                 None => color,
    //                 Some(c) => c,
    //             };
    //         }
    //
    //         // dbg!(color);
    //         write_grey_to_all_screens(&device, color)?;
    //     }
    // }

    // loop {
    //     for i in 0..255 {
    //         write_grey_to_all_screens(&device, i)?;
    //     }
    //
    //     for i in (0..255).rev() {
    //         write_grey_to_all_screens(&device, i)?;
    //     }
    // }

    // device.disconnect()?;

    Ok(())
}

// fn write_grey_to_all_screens(device: &Deck, color: u8) -> eyre::Result<()> {
//     // Make an all white image
//     let mut img = RgbImage::new(72, 72);
//     img.fill(color);
//
//     // Set every button to the white image
//     for button_index in 0..device.kind.key_count() {
//         device.set_button_image(button_index, img.clone().into())?;
//     }
//
//     // // If the device has a screen, create an image for it and set it
//     // if let Some((width, height)) = device.kind.lcd_strip_size() {
//     //     let mut img = RgbImage::new(width as u32, height as u32);
//     //     img.fill(color);
//     //     device.set_lcd_image(img.into())?;
//     // }
//
//     Ok(())
// }
