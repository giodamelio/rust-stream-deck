mod text;

use std::sync::Arc;

use anyhow::{anyhow, ensure, Result};
use async_hid::{AccessMode, Device, DeviceInfo};
use futures_lite::StreamExt;
use image::{codecs::jpeg::JpegEncoder, RgbImage};
use tokio::{
    sync::{mpsc, RwLock},
    task::JoinHandle,
};

use self::text::font_renderer;

pub type SubscriptionResult = (JoinHandle<Result<()>>, mpsc::Receiver<Input>);

#[derive(Clone)]
pub struct StreamDeckPlus {
    device: Arc<RwLock<Device>>,
}

impl std::fmt::Debug for StreamDeckPlus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StreamDeckPlus")
    }
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

        Ok(Self {
            device: Arc::new(RwLock::new(device)),
        })
    }

    pub async fn serial_number(&self) -> Result<String> {
        let mut buffer = [0u8; 32];
        buffer[0] = 0x06;
        let _size = self
            .device
            .read()
            .await
            .read_feature_report(&mut buffer)
            .await?;
        extract_string(&buffer[1..])
    }

    pub async fn firmware_version(&self) -> Result<String> {
        let mut buffer = [0u8; 32];
        buffer[0] = 0x05;
        let _size = self
            .device
            .read()
            .await
            .read_feature_report(&mut buffer)
            .await?;
        // Not sure what the other five bytes of junk is
        extract_string(&buffer[6..])
    }

    pub async fn read_input(&self) -> Result<Input> {
        let mut buffer = [0u8; 14];
        self.device
            .read()
            .await
            .read_input_report(&mut buffer)
            .await?;
        buffer.try_into()
    }

    pub fn subscribe(&self) -> Result<SubscriptionResult> {
        let (tx, rx) = mpsc::channel::<Input>(10);
        let handle = tokio::spawn(subscriber(tx, self.clone()));
        Ok((handle, rx))
    }

    pub async fn set_brightness(&self, percent: u8) -> Result<()> {
        let mut buffer = vec![0x03, 0x08, percent];
        buffer.extend(vec![0u8; 29]);

        self.device
            .write()
            .await
            .write_feature_report(&mut buffer)
            .await?;

        Ok(())
    }

    pub async fn set_button_color(&self, index: u8, color: image::Rgb<u8>) -> Result<()> {
        ensure!(index <= 7, anyhow!("Invalid button index"));

        let img = solid_image(120, 120, color);
        self.set_button_image(index, &img).await
    }

    pub async fn set_button_image(&self, index: u8, image: &RgbImage) -> Result<()> {
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

            self.device.write().await.write_output_report(&buf).await?;

            bytes_remaining -= this_length;
            page_number += 1;
        }

        Ok(())
    }

    pub async fn set_lcd_message(&self, text: String) -> Result<()> {
        let mut renderer = font_renderer().lock().await;
        let img = renderer.render_text(800, 100, text);
        self.set_lcd_image(10, 10, &img).await?;
        Ok(())
    }

    // 800x100 is the dimensions
    pub async fn set_lcd_image(&self, x: u16, y: u16, image: &RgbImage) -> Result<()> {
        ensure!(x <= 800, anyhow!("x must be 800 or less"));
        ensure!(y <= 800, anyhow!("7 must be 100 or less"));

        // Encode it as JPEG
        let (width, height) = image.dimensions();
        let mut image_data = Vec::new();
        let mut encoder = JpegEncoder::new(&mut image_data);
        encoder.encode(image, width, height, image::ExtendedColorType::Rgb8)?;

        // Write the image
        let image_report_length = 1024;
        let image_report_header_length = 16;
        let image_report_payload_length = image_report_length - image_report_header_length;

        let mut page_number = 0;
        let mut bytes_remaining = image_data.len();

        while bytes_remaining > 0 {
            let this_length = bytes_remaining.min(image_report_payload_length);
            let bytes_sent = page_number * image_report_payload_length;

            // Selecting header based on device
            let mut buf: Vec<u8> = vec![
                0x02,
                0x0c,
                (x & 0xff) as u8,
                (x >> 8) as u8,
                (y & 0xff) as u8,
                (y >> 8) as u8,
                (width & 0xff) as u8,
                (width >> 8) as u8,
                (height & 0xff) as u8,
                (height >> 8) as u8,
                if bytes_remaining <= image_report_payload_length {
                    1
                } else {
                    0
                },
                (page_number & 0xff) as u8,
                (page_number >> 8) as u8,
                (this_length & 0xff) as u8,
                (this_length >> 8) as u8,
                0,
            ];

            buf.extend(&image_data[bytes_sent..bytes_sent + this_length]);

            // Adding padding
            buf.extend(vec![0u8; image_report_length - buf.len()]);

            self.device.write().await.write_output_report(&buf).await?;

            bytes_remaining -= this_length;
            page_number += 1;
        }

        Ok(())
    }
}

async fn subscriber(tx: mpsc::Sender<Input>, deck: StreamDeckPlus) -> Result<()> {
    loop {
        let input = deck.read_input().await?;
        tx.send(input).await?;
    }
}

#[derive(Debug)]
pub enum Input {
    None,
    Buttons([bool; 8]),
    EncoderPress([bool; 4]),
    EncoderTwist([i8; 4]),
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
    // Data starts at 4 and continues for the number of buttons
    let values: Vec<bool> = buffer[4..12].iter().map(|b| *b != 0).collect();
    Ok(Input::Buttons([
        values[0], values[1], values[2], values[3], values[4], values[5], values[6], values[7],
    ]))
}

fn read_encoders(buffer: [u8; 14]) -> Result<Input> {
    match buffer[4] {
        // Encoder Press
        // Data starts at 4 and continues for the number of encoders
        0x0 => {
            let values: Vec<bool> = buffer[5..9].iter().map(|b| *b != 0).collect();
            Ok(Input::EncoderPress([
                values[0], values[1], values[2], values[3],
            ]))
        }
        // Encoder Twist
        // Data starts at 4 and continues for the number of encoders
        0x1 => {
            let values: Vec<i8> = buffer[5..9].iter().map(|b| i8::from_le(*b as i8)).collect();
            Ok(Input::EncoderTwist([
                values[0], values[1], values[2], values[3],
            ]))
        }
        _ => Err(anyhow!("Bad Encoder Data")),
    }
}

pub fn solid_image(width: u32, height: u32, color: image::Rgb<u8>) -> RgbImage {
    // Create image of specified color
    let mut img = image::ImageBuffer::new(width, height);
    for pixel in img.pixels_mut() {
        *pixel = color;
    }
    img
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
