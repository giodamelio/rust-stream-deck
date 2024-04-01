mod streamdeck;

use crate::streamdeck::StreamDeckPlus;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::try_init()?;

    let mut deck = StreamDeckPlus::connect_exactly_one().await?;
    tracing::info!("Serial Number: {}", deck.serial_number().await?);

    Ok(())
}
