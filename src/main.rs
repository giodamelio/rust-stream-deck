mod app;
mod streamdeck;

use std::time::Duration;

use anyhow::Result;
use tokio::signal;
use tokio::sync::mpsc;
use tokio::time::sleep;

use crate::app::{spawn_app, Apps};
use crate::streamdeck::{Input, StreamDeckPlus};

async fn app_one(_deck: StreamDeckPlus, mut inputs: mpsc::UnboundedReceiver<Input>) -> Result<()> {
    loop {
        let input = inputs.recv().await;
        tracing::info!("App one got input: {:?}", input);
    }
}

async fn app_two(_deck: StreamDeckPlus, mut inputs: mpsc::UnboundedReceiver<Input>) -> Result<()> {
    loop {
        let input = inputs.recv().await;
        tracing::info!("App two got input: {:?}", input);
    }
}

async fn app_three(
    _deck: StreamDeckPlus,
    mut inputs: mpsc::UnboundedReceiver<Input>,
) -> Result<()> {
    loop {
        let input = inputs.recv().await;
        tracing::info!("App three got input: {:?}", input);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::try_init()?;
    console_subscriber::init();

    // Connect to the StreamDeck
    let deck = StreamDeckPlus::connect_exactly_one().await?;
    tracing::debug!("Serial Number: {}", deck.serial_number().await?);
    tracing::debug!("Firmware Version: {}", deck.firmware_version().await?);

    // Turn it's brightness all the way up
    deck.set_brightness(100).await?;

    // Spin up some apps
    let mut apps = Apps::new(deck.clone());
    spawn_app!(apps, "app one", app_one);
    spawn_app!(apps, "app two", app_two);
    spawn_app!(apps, "app three", app_three);

    // Route the inputs to the active app
    apps.route()?;

    apps.activate(0)?;
    sleep(Duration::from_secs(3)).await;
    apps.activate(1)?;
    sleep(Duration::from_secs(3)).await;
    apps.activate(2)?;

    // Wait for SIGINT to exit
    signal::ctrl_c().await?;
    Ok(())

    // Run the apps
    // loop {
    //     // Print input message
    //     let input = deck.read_input().await?;
    //
    //     if let Input::EncoderPress([_, _, _, true]) = input {
    //         active_app_index = app_picker(&mut deck, &apps, active_app_index).await?;
    //         apps[active_app_index].handle(input).await;
    //     } else {
    //         apps[active_app_index].handle(input).await;
    //     }
    // }

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

// async fn app_picker(
//     deck: &mut StreamDeckPlus,
//     apps: &[Box<dyn App>],
//     active_index: usize,
// ) -> Result<usize> {
//     tracing::debug!("App picker launched");
//
//     let mut new_index = active_index;
//     tracing::trace!("Current App: {:?}", apps[new_index].name());
//     deck.set_lcd_message(format!("App> {}", apps[new_index].name()))
//         .await?;
//     loop {
//         match deck.read_input().await? {
//             Input::EncoderPress([_, _, _, true]) => {
//                 tracing::trace!("App picked: {:?}", apps[new_index].name());
//                 return Ok(new_index);
//             }
//             Input::EncoderTwist([_, _, _, value]) if value != 0 => {
//                 // (Over|Under)flow safe addition of offset
//                 // Couldn't get % to cooperate with me
//                 new_index = match (value.is_negative(), new_index) {
//                     // Underflow
//                     (true, 0) => apps.len() - 1,
//                     // Overflow
//                     (false, val) if val == apps.len() - 1 => 0,
//                     // Decrement
//                     (true, _) => new_index - 1,
//                     // Increment
//                     (false, _) => new_index + 1,
//                 };
//
//                 // Write the name to the screen
//                 deck.set_lcd_message(format!("App> {}", apps[new_index].name()))
//                     .await?;
//             }
//             _other => {}
//         }
//     }
// }
