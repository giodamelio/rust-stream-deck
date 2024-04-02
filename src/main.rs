mod streamdeck;

use crate::streamdeck::StreamDeckPlus;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::try_init()?;

    let mut deck = StreamDeckPlus::connect_exactly_one().await?;
    tracing::info!("Serial Number: {}", deck.serial_number().await?);
    tracing::info!("Serial Number: {}", deck.firmware_version().await?);

    deck.set_brightness(100).await?;

    loop {
        let input = deck.read_input().await?;
        tracing::info!("Input: {:?}", input);
    }
}
