pub mod streamdeck;
mod system_input;

use std::collections::HashMap;

use bevy::asset::HandleId;
use bevy::input::InputSystem;
use bevy::prelude::*;
use bevy::tasks::{IoTaskPool, Task};
use crossbeam_channel::{Receiver, Sender};
use elgato_streamdeck::{info::Kind as RawKind, list_devices, new_hidapi, StreamDeckInput};
use image::DynamicImage;

pub use elgato_streamdeck::StreamDeck as RawStreamDeck;

use crate::streamdeck::{Button, ButtonInput, Command, Encoder};

#[derive(Default)]
pub struct StreamDeckPlugin;

impl Plugin for StreamDeckPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Input<Button>>()
            .init_resource::<Axis<Encoder>>()
            .init_resource::<Input<Encoder>>()
            .init_resource::<ImageCache>()
            .add_event::<Command>()
            .add_event::<ButtonInput>()
            .add_systems(PreStartup, multi_threaded_streamdeck)
            .add_systems(PreUpdate, system_input::inputs.before(InputSystem))
            .add_systems(PreUpdate, forward_commands);
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

fn forward_commands(
    deck_commands: ResMut<StreamDeckCommands>,
    mut ev_command: EventReader<Command>,
) {
    for command in ev_command.iter() {
        deck_commands
            .0
            .send(command.clone())
            .expect("Could not send command");
    }
}

fn multi_threaded_streamdeck(mut commands: Commands) {
    // TODO: these should probably be bounded...
    let (inputs_tx, inputs_rx) = crossbeam_channel::unbounded::<StreamDeckInput>();
    let (commands_tx, commands_rx) = crossbeam_channel::unbounded::<Command>();

    commands.insert_resource(StreamDeckInputs(inputs_rx));
    commands.insert_resource(StreamDeckCommands(commands_tx));

    let pool = IoTaskPool::get();
    let task = pool.spawn(async move {
        let streamdeck = get_device();
        streamdeck.reset().expect("Could not reset streamdeck");
        loop {
            // Handle incoming commands
            if !commands_rx.is_empty() {
                trace!("Commands in channel: {}", commands_rx.len());
            }
            if let Ok(command) = commands_rx.try_recv() {
                trace!("Got command: {:?}", command);
                match command {
                    Command::Shutdown => {
                        streamdeck.reset().expect("Could not reset device");
                    }
                    Command::SetBrightness(brightness) => {
                        streamdeck
                            .set_brightness(brightness)
                            .expect("Could not set brightness");
                    }
                    Command::SetButtonImage(button_index, image) => {
                        streamdeck
                            .set_button_image(button_index, image.clone())
                            .expect("Unable to write button image");
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

    commands.insert_resource(StreamDeckTask(task));
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
