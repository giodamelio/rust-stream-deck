use std::collections::HashMap;
use std::time::Duration;

use anyhow::{anyhow, Result};
use bevy::{
    prelude::{ButtonInput, Commands, Plugin, PreStartup, PreUpdate, Res, ResMut, Resource},
    tasks::IoTaskPool,
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use elgato_streamdeck::{StreamDeck, StreamDeckInput as ELStreamDeckInput};
use tracing::{debug, trace};

pub struct StreamDeckPlugin;

impl Plugin for StreamDeckPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PreStartup, listener)
            .add_systems(PreUpdate, recieve_inputs)
            .init_resource::<ButtonInput<StreamDeckButton>>()
            .init_resource::<ButtonInput<StreamDeckEncoder>>()
            .init_resource::<EncoderPosition>();
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct StreamDeckButton(pub usize);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct StreamDeckEncoder(pub usize);

#[derive(Debug, Default, Resource)]
pub struct EncoderPosition {
    values: HashMap<StreamDeckEncoder, isize>,
}

impl EncoderPosition {
    pub fn get_position(&self, encoder: StreamDeckEncoder) -> isize {
        self.values.get(&encoder).map_or(0, |v| *v)
    }

    pub fn get_position_clamped(
        &self,
        encoder: StreamDeckEncoder,
        min: isize,
        max: isize,
    ) -> isize {
        self.get_position(encoder).clamp(min, max)
    }

    fn update(&mut self, encoder: StreamDeckEncoder, change: i8) {
        let current_position = self.values.get(&encoder).map_or(0, |v| *v);
        self.values
            .insert(encoder, current_position + (change as isize));
    }
}

#[derive(Debug, Resource)]
struct StreamDeckInputListener {
    inputs_rx: Receiver<ELStreamDeckInput>,
}

// This system owns the connection to the StreamDeck
// It communicates to the rest of the application via channels
fn listener(mut commands: Commands) {
    let (inputs_tx, inputs_rx) = unbounded::<ELStreamDeckInput>();

    let taskpool = IoTaskPool::get();
    taskpool.spawn(listener_task(inputs_tx)).detach();

    commands.insert_resource(StreamDeckInputListener { inputs_rx });
}

#[tracing::instrument]
async fn listener_task(inputs_tx: Sender<ELStreamDeckInput>) -> Result<()> {
    let deck = get_exactly_one_streamdeck()?;

    loop {
        match deck.read_input(Some(Duration::from_millis(1)))? {
            // Throw away no data events
            ELStreamDeckInput::NoData => (),
            input => inputs_tx.try_send(input)?,
        };
    }
}

// Convert input events from the StreamDeck to Bevy ButtonInput
fn recieve_inputs(
    input_listener: Res<StreamDeckInputListener>,
    mut buttons: ResMut<ButtonInput<StreamDeckButton>>,
    mut encoders: ResMut<ButtonInput<StreamDeckEncoder>>,
    mut encoders_positions: ResMut<EncoderPosition>,
) {
    buttons.clear();
    encoders.clear();

    for input in input_listener.inputs_rx.try_iter() {
        trace!("Recieved input: {:?}", input);

        match input {
            ELStreamDeckInput::ButtonStateChange(new_button_states) => {
                for (index, state) in new_button_states.iter().enumerate() {
                    let key = StreamDeckButton(index);
                    match (*state, buttons.pressed(key)) {
                        // If it is pressed and not already pressed, press it
                        (true, false) => buttons.press(key),
                        // If it is not pressed, and not already relased, release it
                        (false, true) => buttons.release(key),
                        // Otherwise the state stayed the same and we can ignore it
                        _ => {}
                    }
                }
            }
            ELStreamDeckInput::EncoderStateChange(new_encoder_states) => {
                for (index, state) in new_encoder_states.iter().enumerate() {
                    let encoder = StreamDeckEncoder(index);
                    match (*state, encoders.pressed(encoder)) {
                        // If it is pressed and not already pressed, press it
                        (true, false) => encoders.press(encoder),
                        // If it is not pressed, and not already relased, release it
                        (false, true) => encoders.release(encoder),
                        // Otherwise the state stayed the same and we can ignore it
                        _ => {}
                    }
                }
            }
            ELStreamDeckInput::EncoderTwist(changes) => {
                for (index, change) in changes.iter().enumerate() {
                    encoders_positions.update(StreamDeckEncoder(index), *change);
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
