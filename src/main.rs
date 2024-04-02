mod app;
mod streamdeck;

use anyhow::Result;
use futures_lite::future::Boxed;

use crate::app::App;
use crate::streamdeck::{Input, StreamDeckPlus};

struct AppBrightness;

impl App for AppBrightness {
    fn name(&self) -> &'static str {
        "Brightness"
    }

    fn handle(&self, input: Input) -> Boxed<()> {
        Box::pin(async move {
            tracing::info!("Brightness got input: {:?}", input);
        })
    }
}

struct AppRandomColor;

impl App for AppRandomColor {
    fn name(&self) -> &'static str {
        "Random Color"
    }

    fn handle(&self, input: Input) -> Boxed<()> {
        Box::pin(async move {
            tracing::info!("RandomColor got input: {:?}", input);
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::try_init()?;

    // Connect to the StreamDeck
    let mut deck = StreamDeckPlus::connect_exactly_one().await?;
    tracing::info!("Serial Number: {}", deck.serial_number().await?);
    tracing::info!("Serial Number: {}", deck.firmware_version().await?);

    // Turn it's brightness all the way up
    deck.set_brightness(100).await?;

    // Init the apps
    let apps: Vec<Box<dyn App>> = vec![Box::new(AppBrightness), Box::new(AppRandomColor)];
    let mut active_app_index = 0;

    // Run first active app once
    apps[active_app_index].handle(Input::None).await;

    // Run the apps
    loop {
        // Print input message
        let input = deck.read_input().await?;

        if let Input::EncoderPress([_, _, _, true]) = input {
            active_app_index = app_picker(&mut deck, &apps, active_app_index).await?;
            apps[active_app_index].handle(input).await;
        } else {
            apps[active_app_index].handle(input).await;
        }
    }

    // let mut count = 0;
    // loop {
    //     let color = image::Rgb::<u8>([rand::random(), rand::random(), rand::random()]);
    //
    //     // Draw some text to the LCD
    //     deck.set_lcd_message(format!("Count: {}", count)).await?;
    //
    //     // Set the first button a random color
    //     deck.set_button_color(0, color).await?;
    //
    //     // Add a shape to the LCD of the same color
    //     let img = streamdeck::solid_image(50, 50, color);
    //     deck.set_lcd_image(725, 25, &img).await?;
    //
    //     // Print input message
    //     let input = deck.read_input().await?;
    //     tracing::info!("Input: {:?}", input);
    //
    //     // Update the count
    //     if let Input::EncoderTwist([value, _, _, _]) = input {
    //         count += value as i32;
    //     }
    // }
}

async fn app_picker(
    deck: &mut StreamDeckPlus,
    apps: &[Box<dyn App>],
    active_index: usize,
) -> Result<usize> {
    tracing::debug!("App picker launched");

    let mut new_index = active_index;
    tracing::trace!("Current App: {:?}", apps[new_index].name());
    deck.set_lcd_message(format!("App> {}", apps[new_index].name()))
        .await?;
    loop {
        match deck.read_input().await? {
            Input::EncoderPress([_, _, _, true]) => {
                tracing::trace!("App picked: {:?}", apps[new_index].name());
                return Ok(new_index);
            }
            Input::EncoderTwist([_, _, _, value]) if value != 0 => {
                // (Over|Under)flow safe addition of offset
                // Couldn't get % to cooperate with me
                new_index = match (value.is_negative(), new_index) {
                    // Underflow
                    (true, 0) => apps.len() - 1,
                    // Overflow
                    (false, val) if val == apps.len() - 1 => 0,
                    // Decrement
                    (true, _) => new_index - 1,
                    // Increment
                    (false, _) => new_index + 1,
                };

                // Write the name to the screen
                deck.set_lcd_message(format!("App> {}", apps[new_index].name()))
                    .await?;
            }
            _other => {}
        }
    }
}
