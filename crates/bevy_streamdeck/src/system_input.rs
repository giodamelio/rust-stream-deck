use bevy::input::{Axis, Input};
use bevy::log::trace;
use bevy::prelude::{NonSendMut, ResMut};
use elgato_streamdeck::{StreamDeck as RawStreamDeck, StreamDeckInput};

use crate::{StreamDeckButton, StreamDeckEncoder};

// Handle input events from the StreamDeck and convert them to Input<T>
pub fn system(
    streamdeck: NonSendMut<RawStreamDeck>,
    mut button_inputs: ResMut<Input<StreamDeckButton>>,
    mut encoder_axis: ResMut<Axis<StreamDeckEncoder>>,
    mut encoder_inputs: ResMut<Input<StreamDeckEncoder>>,
) {
    // Clear all the events
    button_inputs.clear();
    encoder_inputs.clear();

    // Handle incoming events
    if let Ok(event) = streamdeck.read_input(None) {
        match event {
            StreamDeckInput::NoData => return,
            StreamDeckInput::ButtonStateChange(buttons) => {
                trace!("Button state change: {:?}", buttons);

                for (index, button_pressed) in buttons.iter().enumerate() {
                    let button = StreamDeckButton(index as u8);

                    // If the input is currently pressed, and event is not pressed, release the input
                    if button_inputs.pressed(button) && *button_pressed == false {
                        button_inputs.release(button);
                        continue;
                    }

                    // If the button is not pressed, and the event says that is is, press the input
                    if !button_inputs.pressed(button) && *button_pressed == true {
                        button_inputs.press(button);
                        continue;
                    }
                }
            }
            StreamDeckInput::EncoderStateChange(encoders) => {
                trace!("Encoder state change: {:?}", encoders);

                for (index, encoder_pressed) in encoders.iter().enumerate() {
                    let encoder = StreamDeckEncoder(index as u8);

                    // If the input is currently pressed, and event is not pressed, release the input
                    if encoder_inputs.pressed(encoder) && *encoder_pressed == false {
                        encoder_inputs.release(encoder);
                        continue;
                    }

                    // If the button is not pressed, and the event says that is is, press the input
                    if !encoder_inputs.pressed(encoder) && *encoder_pressed == true {
                        encoder_inputs.press(encoder);
                        continue;
                    }
                }
            }
            StreamDeckInput::EncoderTwist(encoders) => {
                trace!("Encoder twist: {:?}", encoders);

                for (index, change) in encoders.iter().enumerate() {
                    let knob = StreamDeckEncoder(index as u8);

                    let current = encoder_axis.get_unclamped(knob).unwrap_or(0.0);
                    encoder_axis.set(knob, current + (*change as f32));
                }
            }
            StreamDeckInput::TouchScreenPress(x, y) => {
                trace!("Touch screen press x={} y={}", x, y);
            }
            StreamDeckInput::TouchScreenLongPress(x, y) => {
                trace!("Touch screen long press x={} y={}", x, y);
            }
            StreamDeckInput::TouchScreenSwipe((startx, starty), (endx, endy)) => {
                trace!(
                    "Touch screen long press start={},{} end={},{}",
                    startx,
                    starty,
                    endx,
                    endy
                );
            }
        }
    }
}
