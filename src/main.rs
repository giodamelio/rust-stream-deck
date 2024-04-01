use anyhow::Result;
use async_hid::{AccessMode, DeviceInfo, SerialNumberExt};
use futures_lite::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::try_init()?;

    let mut devices = DeviceInfo::enumerate().await?;
    while let Some(device) = devices.next().await {
        println!("Device: {:?}", device);
        println!("Device: {:?}", device.serial_number());
    }

    let device = DeviceInfo::enumerate()
        .await?
        // StreamDeck Plus
        .find(|info: &DeviceInfo| info.matches(12, 1, 4057, 132))
        .await
        .expect("Could not find device")
        .open(AccessMode::ReadWrite)
        .await?;

    let mut buffer = [0u8; 32];
    buffer[0] = 0x06;
    let size = device.read_feature_report(&mut buffer).await?;
    tracing::info!("Size: {}, Data: {:?}", size, &buffer[..size]);
    tracing::info!("Serial?: {}", extract_string(&buffer[1..])?);

    // Get Info
    tracing::info!("Info: {:?}", device.info());

    Ok(())
}

fn extract_string(bytes: &[u8]) -> Result<String> {
    // Find the position of the last non-NUL byte
    let last_non_nul_pos = bytes.iter().rposition(|&x| x != 0x00);

    // Determine the slice up to the last non-NUL byte (or the whole slice if no NUL bytes)
    let trimmed_slice = match last_non_nul_pos {
        Some(pos) => &bytes[..=pos],
        None => bytes,
    };

    // Convert the trimmed slice to a Vec<u8> because String::from_utf8 expects Vec<u8>
    let trimmed_vec = trimmed_slice.to_vec();
    Ok(String::from_utf8(trimmed_vec)?)
}
