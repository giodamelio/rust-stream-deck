use bevy::log::{self, LogPlugin};
use bevy::prelude::*;

use bevy_streamdeck::{
    StreamDeckBrightness, StreamDeckButton, StreamDeckEncoder, StreamDeckPlugin,
};

fn main() {
    App::new()
        // Add the minimal plugins and the logger
        .add_plugins((
            MinimalPlugins,
            LogPlugin {
                filter: "bevy_hid_experiment=trace,bevy_streamdeck=debug".into(),
                level: log::Level::DEBUG,
            },
        ))
        // StreamDeck Plugin
        .add_plugins(StreamDeckPlugin)
        .add_systems(Update, log_button_presses)
        .add_systems(Update, log_encoder_twists)
        .add_systems(Update, log_encoder_presses)
        // Blink the backlight every 2 seconds
        .add_systems(Update, change_backlight_level)
        .run();
}

fn log_button_presses(buttons: Res<Input<StreamDeckButton>>) {
    if buttons.just_pressed(StreamDeckButton(0)) {
        info!("Button 0 just pressed!");
    }
}

fn log_encoder_twists(encoders: Res<Axis<StreamDeckEncoder>>) {
    if encoders.is_changed() {
        info!(
            "Knob 0 value: {:?}",
            encoders.get_unclamped(StreamDeckEncoder(0))
        );
    }
}

fn log_encoder_presses(encoders: Res<Input<StreamDeckEncoder>>) {
    if encoders.just_pressed(StreamDeckEncoder(0)) {
        info!("Encoder 0 just pressed!");
    }
}

fn change_backlight_level(
    mut brightness: ResMut<StreamDeckBrightness>,
    encoders: Res<Axis<StreamDeckEncoder>>,
) {
    if encoders.is_changed() {
        let new_brightness = encoders
            .get_unclamped(StreamDeckEncoder(0))
            .unwrap_or(0.0)
            .clamp(0.0, 100.0);

        *brightness = StreamDeckBrightness(new_brightness as u8);
    }
}
