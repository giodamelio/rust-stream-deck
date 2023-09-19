mod streamdeck;
mod system_input;

use bevy::input::InputSystem;
use bevy::prelude::*;
use elgato_streamdeck::{info::Kind as RawKind, list_devices, new_hidapi};

pub use elgato_streamdeck::StreamDeck as RawStreamDeck;
pub use streamdeck::{StreamDeckBrightness, StreamDeckButton, StreamDeckEncoder, StreamDeckEvent};

#[derive(Default)]
pub struct StreamDeckPlugin;

impl Plugin for StreamDeckPlugin {
    fn build(&self, app: &mut App) {
        let deck = get_device();
        app.init_resource::<Input<StreamDeckButton>>()
            .init_resource::<Axis<StreamDeckEncoder>>()
            .init_resource::<Input<StreamDeckEncoder>>()
            .init_resource::<StreamDeckBrightness>()
            .insert_non_send_resource(deck)
            .add_event::<StreamDeckEvent>()
            .add_systems(PreUpdate, system_input::system.before(InputSystem))
            .add_systems(Update, system_backlight)
            .add_systems(Update, system_event);
    }
}

fn system_backlight(streamdeck: NonSendMut<RawStreamDeck>, brightness: Res<StreamDeckBrightness>) {
    if brightness.is_changed() {
        streamdeck
            .set_brightness(brightness.0)
            .expect("Could not set backlight brightness");
    }
}

fn system_event(
    streamdeck: NonSendMut<RawStreamDeck>,
    mut events: EventReader<StreamDeckEvent>,
    images: Res<Assets<Image>>,
) {
    for event in events.iter() {
        match event {
            StreamDeckEvent::ButtonSetImage(handle) => {
                if let Some(image) = images.get(handle) {
                    let i = image.clone().try_into_dynamic().unwrap();
                    streamdeck.set_button_image(0, i).unwrap();
                }
            }
        }
    }
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
