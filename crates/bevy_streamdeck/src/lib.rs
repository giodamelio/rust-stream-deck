pub mod streamdeck;
mod system_input;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use bevy::asset::{HandleId, LoadState};
use bevy::input::InputSystem;
use bevy::prelude::*;
use bevy::tasks::{IoTaskPool, Task};
use crossbeam_channel::{Receiver, Sender};
use elgato_streamdeck::{info::Kind as RawKind, list_devices, new_hidapi, StreamDeckInput};
use image::DynamicImage;

pub use elgato_streamdeck::StreamDeck as RawStreamDeck;

use crate::streamdeck::Command;
use streamdeck::{Brightness, Button, ButtonImage, Encoder};

#[derive(Default)]
pub struct StreamDeckPlugin;

impl Plugin for StreamDeckPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Input<Button>>()
            .init_resource::<Axis<Encoder>>()
            .init_resource::<Input<Encoder>>()
            .init_resource::<Brightness>()
            .init_resource::<ButtonImage>()
            .init_resource::<ImageCache>()
            .add_systems(PreStartup, multi_threaded_streamdeck)
            // .add_systems(PreUpdate, system_input::inputs.before(InputSystem))
            .add_systems(Update, system_backlight)
            .add_systems(Update, system_button_image);
    }
}

fn system_backlight(deck_commands: ResMut<StreamDeckCommands>, brightness: Res<Brightness>) {
    // if brightness.is_changed() {
    trace!("Setting brightness");
    deck_commands
        // .send(Command::SetBrightness(brightness.0))
        .send(Command::SetBrightness(0))
        .expect("Could not set backlight brightness");
    // }
}

fn system_button_image(
    deck_commands: ResMut<StreamDeckCommands>,
    button_image: Res<ButtonImage>,
    asset_server: Res<AssetServer>,
) {
    if button_image.is_changed() {
        let ButtonImage(index, handle) = button_image.into_inner();
        deck_commands
            .send(Command::SetButtonImage(*index, handle.clone()))
            .expect("Could not set backlight brightness");
    }
}

#[derive(Resource, Debug)]
pub struct StreamDeckInputs(pub Receiver<StreamDeckInput>);

#[derive(Resource, Deref, Debug)]
struct StreamDeckCommands(Sender<Command>);

#[derive(Resource, Debug)]
struct StreamDeckTask(Task<()>);

#[derive(Resource, Deref, DerefMut, Debug, Default)]
struct ImageCache(HashMap<HandleId, DynamicImage>);

fn multi_threaded_streamdeck(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    asset_server: Res<AssetServer>,
    mut image_cache: ResMut<ImageCache>,
) {
    // TODO: these should probably be bounded...
    let (inputs_tx, inputs_rx) = crossbeam_channel::unbounded::<StreamDeckInput>();
    let (commands_tx, commands_rx) = crossbeam_channel::unbounded::<Command>();
    let inner_commands_tx = commands_tx.clone();

    // commands_tx
    //     .clone()
    //     .send(Command::SetBrightness(0))
    //     .expect("TODO: panic message");

    // commands.insert_resource(StreamDeckTask());
    commands.insert_resource(StreamDeckInputs(inputs_rx));
    commands.insert_resource(StreamDeckCommands(commands_tx));

    let pool = IoTaskPool::get();
    let _results = pool.spawn(async move {
        let streamdeck = get_device();
        loop {
            // Handle incoming commands
            if !commands_rx.is_empty() {
                trace!("Commands in channel: {}", commands_rx.len());
            }
            if let Ok(command) = commands_rx.try_recv() {
                trace!("Got command: {:?}", command);
                match command {
                    Command::SetBrightness(brightness) => {
                        streamdeck
                            .set_brightness(brightness)
                            .expect("Could not set brightness");
                    }
                    Command::SetButtonImage(button_index, image_handle) => {
                        // Get the image from the cache, otherwise convert it and save to cache
                        match image_cache.get(&image_handle.id()) {
                            Some(image) => {
                                trace!("Image cache hit");

                                streamdeck
                                    .set_button_image(button_index, image.clone())
                                    .expect("Unable to write button image");
                            }
                            None => {
                                trace!("Image cache miss");

                                loop {
                                    if asset_server.get_load_state(image_handle.clone())
                                        == LoadState::Loaded
                                    {
                                        break;
                                    }
                                }

                                // Convert image
                                let image = images
                                    .get(&image_handle)
                                    .expect("Image already loaded")
                                    .clone()
                                    .try_into_dynamic()
                                    .expect("Could not convert image");

                                image_cache.insert(image_handle.id(), image);

                                // Add it to the image cache
                                inner_commands_tx
                                    .send(Command::SetButtonImage(button_index, image_handle))
                                    .expect("Could not send command");
                            }
                        };
                    }
                }
            }

            // Handle input events
            if let Ok(event) = streamdeck.read_input(None) {
                if let StreamDeckInput::NoData = event {
                    continue;
                }
                trace!("Got Input: {:?}", event);
                inputs_tx.send(event).unwrap();
            }
        }
    });
}

// Get the StreamDeck device
// TODO: work with more then a single device
fn get_device() -> RawStreamDeck {
    let hid = new_hidapi().expect("Could get HID API");
    let mut devices: Vec<(RawKind, String)> = list_devices(&hid);
    if devices.len() != 1 {
        error!("More then 1 StreamDeck device not supported currently");
        panic!();
    }
    let (kind, serial) = devices.remove(0);
    debug!(
        "StreamDeck of kind={:?} on serial={} selected",
        kind, serial
    );
    let deck = RawStreamDeck::connect(&hid, kind, &serial).unwrap();
    debug!(
        "Connected to StreamDeck kind={:?} serial_number={:?}",
        deck.kind(),
        deck.serial_number()
    );

    deck
}
