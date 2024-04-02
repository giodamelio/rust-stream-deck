use anyhow::{anyhow, ensure, Result};
use async_hid::{AccessMode, Device, DeviceInfo};
use futures_lite::StreamExt;
use image::{codecs::jpeg::JpegEncoder, ColorType, RgbImage};

pub struct StreamDeckPlus {
    device: Device,
}

#[allow(dead_code)]
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

    pub async fn read_input(&mut self) -> Result<Input> {
        let mut buffer = [0u8; 14];
        self.device.read_input_report(&mut buffer).await?;
        buffer.try_into()
    }

    pub async fn set_brightness(&mut self, percent: u8) -> Result<()> {
        let mut buffer = vec![0x03, 0x08, percent];
        buffer.extend(vec![0u8; 29]);

        self.device.write_feature_report(&mut buffer).await?;

        Ok(())
    }

    pub async fn set_button_color(&mut self, index: u8, color: image::Rgb<u8>) -> Result<()> {
        ensure!(index <= 7, anyhow!("Invalid button index"));

        // Create image of specified color
        let mut img = image::ImageBuffer::new(120, 120);
        for pixel in img.pixels_mut() {
            *pixel = color;
        }

        self.set_button_image(index, &img).await
    }

    pub async fn set_button_image(&mut self, index: u8, image: &RgbImage) -> Result<()> {
        ensure!(index <= 7, anyhow!("Invalid button index"));

        // Encode it as JPEG
        let mut image_data = Vec::new();
        let mut encoder = JpegEncoder::new(&mut image_data);
        encoder.encode(image, 120, 120, image::ExtendedColorType::Rgb8)?;

        // Write the image
        let image_report_length = 1024;
        let image_report_header_length = 8;
        let image_report_payload_length = image_report_length - image_report_header_length;

        let mut page_number = 0;
        let mut bytes_remaining = image_data.len();

        while bytes_remaining > 0 {
            let this_length = bytes_remaining.min(image_report_payload_length);
            let bytes_sent = page_number * image_report_payload_length;

            // Selecting header based on device
            let mut buf: Vec<u8> = vec![
                0x02,
                0x07,
                index,
                if this_length == bytes_remaining { 1 } else { 0 },
                (this_length & 0xff) as u8,
                (this_length >> 8) as u8,
                (page_number & 0xff) as u8,
                (page_number >> 8) as u8,
            ];

            buf.extend(&image_data[bytes_sent..bytes_sent + this_length]);

            // Adding padding
            buf.extend(vec![0u8; image_report_length - buf.len()]);

            self.device.write_output_report(&buf).await?;

            bytes_remaining -= this_length;
            page_number += 1;
        }

        Ok(())
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
