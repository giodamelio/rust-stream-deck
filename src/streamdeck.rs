use std::time::Duration;

use anyhow::{anyhow, Result};
use bevy::{
    prelude::{ButtonInput, Commands, Plugin, PreStartup, PreUpdate, Res, ResMut, Resource},
    tasks::IoTaskPool,
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use elgato_streamdeck::{StreamDeck, StreamDeckInput};
use tracing::{debug, trace};

pub struct StreamDeckPlugin;

impl Plugin for StreamDeckPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PreStartup, listener)
            .add_systems(PreUpdate, recieve_inputs)
            .init_resource::<ButtonInput<StreamDeckButton>>();
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum StreamDeckButton {
    Button(i8),
    Encoder(i8),
}

#[derive(Debug, Resource)]
struct StreamDeckInputListener {
    inputs_rx: Receiver<StreamDeckInput>,
}

// This system owns the connection to the StreamDeck
// It communicates to the rest of the application via channels
fn listener(mut commands: Commands) {
    let (inputs_tx, inputs_rx) = unbounded::<StreamDeckInput>();

    let taskpool = IoTaskPool::get();
    taskpool.spawn(listener_task(inputs_tx)).detach();

    commands.insert_resource(StreamDeckInputListener { inputs_rx });
}

#[tracing::instrument]
async fn listener_task(inputs_tx: Sender<StreamDeckInput>) -> Result<()> {
    let deck = get_exactly_one_streamdeck()?;

    loop {
        match deck.read_input(Some(Duration::from_millis(1)))? {
            // Throw away no data events
            StreamDeckInput::NoData => (),
            input => inputs_tx.try_send(input)?,
        };
    }
}

// Convert input events from the StreamDeck to Bevy ButtonInput
fn recieve_inputs(
    input_listener: Res<StreamDeckInputListener>,
    mut inputs: ResMut<ButtonInput<StreamDeckButton>>,
) {
    inputs.clear();

    for input in input_listener.inputs_rx.try_iter() {
        trace!("Recieved input: {:?}", input);

        match input {
            StreamDeckInput::ButtonStateChange(buttons) => {
                for (index, state) in buttons.iter().enumerate() {
                    let key = StreamDeckButton::Button(index as i8);
                    match (*state, inputs.pressed(key)) {
                        // If it is pressed and not already pressed, press it
                        (true, false) => inputs.press(key),
                        // If it is not pressed, and not already relased, release it
                        (false, true) => inputs.release(key),
                        // Otherwise the state stayed the same and we can ignore it
                        _ => {}
                    }
                }
            }
            StreamDeckInput::EncoderStateChange(knobs) => {
                for (index, state) in knobs.iter().enumerate() {
                    let knob = StreamDeckButton::Encoder(index as i8);
                    match (*state, inputs.pressed(knob)) {
                        // If it is pressed and not already pressed, press it
                        (true, false) => inputs.press(knob),
                        // If it is not pressed, and not already relased, release it
                        (false, true) => inputs.release(knob),
                        // Otherwise the state stayed the same and we can ignore it
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

// Get the StreamDeck device
// Error if there is more then one available
fn get_exactly_one_streamdeck() -> Result<StreamDeck> {
    let hid = elgato_streamdeck::new_hidapi()?;
    let devices = elgato_streamdeck::list_devices(&hid);

    let (kind, serial) = devices
        .first()
        .ok_or(anyhow!("Only exactly one StreamDeck at a time"))?;

    debug!("Found StreamDeck, Kind: {:?}, Serial: {:?}", kind, serial);

    let device = StreamDeck::connect(&hid, *kind, serial)?;

    debug!(
        "Connected to StreamDeck, Firmware Version: {}",
        device.firmware_version()?
    );

    Ok(device)
}
