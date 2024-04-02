mod streamdeck;

use crate::streamdeck::{Input, StreamDeckPlus};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::try_init()?;

    let mut deck = StreamDeckPlus::connect_exactly_one().await?;
    tracing::info!("Serial Number: {}", deck.serial_number().await?);
    tracing::info!("Serial Number: {}", deck.firmware_version().await?);

    deck.set_brightness(100).await?;

    let mut count: i32 = 0;
    loop {
        let color = image::Rgb::<u8>([rand::random(), rand::random(), rand::random()]);

        // Draw some text to the LCD
        deck.set_lcd_message(format!("Count: {}", count)).await?;

        // Set the first button a random color
        deck.set_button_color(0, color).await?;

        // Add a shape to the LCD of the same color
        let img = streamdeck::solid_image(50, 50, color);
        deck.set_lcd_image(725, 25, &img).await?;

        // Print input message
        let input = deck.read_input().await?;
        tracing::info!("Input: {:?}", input);

        // Update the count
        if let Input::EncoderTwist([value, _, _, _]) = input {
            count += value as i32;
        }
    }
}
