use anyhow::{anyhow, Result};
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
        // Not sure what the other five bytes of junk is
        extract_string(&buffer[6..])
    }

    pub async fn set_brightness(&mut self, percent: u8) -> Result<()> {
        let mut buffer = vec![0x03, 0x08, percent];
        buffer.extend(vec![0u8; 29]);

        self.device.write_feature_report(&mut buffer).await?;

        Ok(())
    }

    pub async fn read_input(&mut self) -> Result<Input> {
        let mut buffer = [0u8; 14];
        self.device.read_input_report(&mut buffer).await?;
        buffer.try_into()
    }
}

#[derive(Debug)]
pub enum Input {
    None,
    Buttons(Vec<bool>),
    EncoderPress(Vec<bool>),
    EncoderTwist(Vec<i8>),
}

impl TryFrom<[u8; 14]> for Input {
    type Error = anyhow::Error;

    fn try_from(buffer: [u8; 14]) -> std::prelude::v1::Result<Self, Self::Error> {
        if buffer[0] == 0x0 {
            return Ok(Input::None);
        }

        match buffer[1] {
            0x0 => read_buttons(buffer),
            // 0x2 => {}, // LCD Input
            0x3 => read_encoders(buffer),
            byte => Err(anyhow!("Unknown data type: {}", byte)),
        }
    }
}

fn read_buttons(buffer: [u8; 14]) -> Result<Input> {
    Ok(Input::Buttons(
        // Data starts at 4 and continues for the number of buttons
        buffer[4..12].iter().map(|b| *b != 0).collect(),
    ))
}

fn read_encoders(buffer: [u8; 14]) -> Result<Input> {
    match buffer[4] {
        // Encoder Press
        // Data starts at 4 and continues for the number of encoders
        0x0 => Ok(Input::EncoderPress(
            buffer[5..9].iter().map(|b| *b != 0).collect(),
        )),
        // Encoder Twist
        // Data starts at 4 and continues for the number of encoders
        0x1 => Ok(Input::EncoderTwist(
            buffer[5..9].iter().map(|b| i8::from_le(*b as i8)).collect(),
        )),
        _ => Err(anyhow!("Bad Encoder Data")),
    }
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
