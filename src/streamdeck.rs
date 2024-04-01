use anyhow::Result;
use async_hid::{AccessMode, Device, DeviceInfo};
use futures_lite::StreamExt;

pub struct StreamDeckPlus {
    device: Device,
}

impl StreamDeckPlus {
    pub async fn connect_exactly_one() -> Result<Self> {
        let device = DeviceInfo::enumerate()
            .await?
            // StreamDeck Plus
            .find(|info: &DeviceInfo| info.matches(12, 1, 4057, 132))
            .await
            .expect("Could not find device")
            .open(AccessMode::ReadWrite)
            .await?;

        Ok(Self { device })
    }

    pub async fn serial_number(&mut self) -> Result<String> {
        let mut buffer = [0u8; 32];
        buffer[0] = 0x06;
        let _size = self.device.read_feature_report(&mut buffer).await?;
        extract_string(&buffer[1..])
    }

    pub async fn firmware_version(&mut self) -> Result<String> {
        let mut buffer = [0u8; 32];
        buffer[0] = 0x05;
        let _size = self.device.read_feature_report(&mut buffer).await?;
        dbg!(buffer);
        // Not sure what the other five bytes of junk is
        extract_string(&buffer[6..])
    }
}

pub fn extract_string(bytes: &[u8]) -> Result<String> {
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
