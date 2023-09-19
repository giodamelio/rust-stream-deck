use bevy::log::{self, LogPlugin};
use bevy::prelude::*;
use std::collections::HashMap;

use bevy_streamdeck::{
    StreamDeckBrightness, StreamDeckButton, StreamDeckEncoder, StreamDeckEvent, StreamDeckPlugin,
};

fn main() {
    App::new()
        // Add some plugins
        .add_plugins((
            MinimalPlugins,
            LogPlugin {
                filter: "bevy_hid_experiment=trace,bevy_streamdeck=debug".into(),
                level: log::Level::DEBUG,
            },
            AssetPlugin::default(),
            ImagePlugin::default(),
        ))
        // StreamDeck Plugin
        .add_plugins(StreamDeckPlugin)
        // Load assets
        .insert_resource(ImageCatalog::default())
        .add_systems(Startup, load_assets)
        // Some Debug loggers
        .add_systems(Update, log_button_presses)
        .add_systems(Update, log_encoder_twists)
        .add_systems(Update, log_encoder_presses)
        // Change the backlight level with a knob
        .add_systems(Update, change_backlight_level)
        // Set a button image on the streamdeck
        .add_systems(Update, set_button_pumpkin)
        .run();
}

#[derive(Resource, Default)]
struct ImageCatalog {
    handles: HashMap<&'static str, Handle<Image>>,
}

fn load_assets(asset_server: Res<AssetServer>, mut image_catalog: ResMut<ImageCatalog>) {
    let pumpkin_handle: Handle<Image> = asset_server.load("pumpkin.png");
    image_catalog.handles.insert("pumpkin", pumpkin_handle);
}

fn set_button_pumpkin(
    image_catalog: Res<ImageCatalog>,
    mut ev_streamdeck: EventWriter<StreamDeckEvent>,
) {
    if let Some(pumpkin_handle) = image_catalog.handles.get("pumpkin") {
        ev_streamdeck.send(StreamDeckEvent::ButtonSetImage(pumpkin_handle.clone()))
    }
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
            .unwrap_or(100.0)
            .clamp(0.0, 100.0);

        *brightness = StreamDeckBrightness(new_brightness as u8);
    }
}
