use bevy::input::{Axis, ButtonState, Input};
use bevy::log::trace;
use bevy::prelude::*;
use elgato_streamdeck::StreamDeckInput;

use crate::streamdeck::{Button, ButtonInput, Encoder};
use crate::StreamDeckInputs;

// Handle input events from the StreamDeck and convert them to Input<T>
pub fn inputs(
    inputs: Res<StreamDeckInputs>,
    mut button_inputs: ResMut<Input<Button>>,
    mut encoder_axis: ResMut<Axis<Encoder>>,
    mut encoder_inputs: ResMut<Input<Encoder>>,
    mut ev_buttoninput: EventWriter<ButtonInput>,
) {
    // Clear all the events
    button_inputs.clear();
    encoder_inputs.clear();

    for input in inputs.0.try_iter() {
        match input {
            StreamDeckInput::NoData => (),
            StreamDeckInput::ButtonStateChange(buttons) => {
                for (index, button_pressed) in buttons.iter().enumerate() {
                    let button = Button(index as u8);

                    // If the input is currently pressed, and event is not pressed, release the input
                    if button_inputs.pressed(button) && !(*button_pressed) {
                        button_inputs.release(button);
                        ev_buttoninput.send(ButtonInput::new(button, ButtonState::Released));
                        continue;
                    }

                    // If the button is not pressed, and the event says that is is, press the input
                    if !button_inputs.pressed(button) && *button_pressed {
                        button_inputs.press(button);
                        ev_buttoninput.send(ButtonInput::new(button, ButtonState::Pressed));
                        continue;
                    }
                }
            }
            StreamDeckInput::EncoderStateChange(encoders) => {
                for (index, encoder_pressed) in encoders.iter().enumerate() {
                    let encoder = Encoder(index as u8);

                    // If the input is currently pressed, and event is not pressed, release the input
                    if encoder_inputs.pressed(encoder) && !(*encoder_pressed) {
                        encoder_inputs.release(encoder);
                        continue;
                    }

                    // If the button is not pressed, and the event says that is is, press the input
                    if !encoder_inputs.pressed(encoder) && *encoder_pressed {
                        encoder_inputs.press(encoder);
                        continue;
                    }
                }
            }
            StreamDeckInput::EncoderTwist(encoders) => {
                for (index, change) in encoders.iter().enumerate() {
                    let knob = Encoder(index as u8);

                    let current = encoder_axis.get_unclamped(knob).unwrap_or(0.0);
                    encoder_axis.set(knob, current + (*change as f32));
                }
            }
            // TODO: implement this
            StreamDeckInput::TouchScreenPress(x, y) => {
                trace!("Touch screen press x={} y={}", x, y);
            }
            // TODO: implement this
            StreamDeckInput::TouchScreenLongPress(x, y) => {
                trace!("Touch screen long press x={} y={}", x, y);
            }
            // TODO: implement this
            StreamDeckInput::TouchScreenSwipe((start_x, start_y), (end_x, end_y)) => {
                trace!(
                    "Touch screen long press start={},{} end={},{}",
                    start_x,
                    start_y,
                    end_x,
                    end_y
                );
            }
        }
    }
}
