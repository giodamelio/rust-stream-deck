mod streamdeck;

use crate::streamdeck::StreamDeckPlus;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::try_init()?;

    let mut deck = StreamDeckPlus::connect_exactly_one().await?;
    tracing::info!("Serial Number: {}", deck.serial_number().await?);
    tracing::info!("Serial Number: {}", deck.firmware_version().await?);

    deck.set_brightness(100).await?;

    // Set the first button a random color
    let color = image::Rgb::<u8>([rand::random(), rand::random(), rand::random()]);
    deck.set_button_color(0, color).await?;

    // Add a shape to the LCD of the same color
    let img = streamdeck::solid_image(50, 50, color);
    deck.set_lcd_image(25, 25, &img).await?;

    loop {
        let input = deck.read_input().await?;
        tracing::info!("Input: {:?}", input);

        // Set the first button a random color
        let color = image::Rgb::<u8>([rand::random(), rand::random(), rand::random()]);
        deck.set_button_color(0, color).await?;
    }
}
