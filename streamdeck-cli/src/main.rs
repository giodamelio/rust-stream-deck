use bevy::log::{self, LogPlugin};
use bevy::prelude::*;
use std::collections::HashMap;

use bevy_streamdeck::{streamdeck, StreamDeckPlugin};

fn main() {
    App::new()
        // Add some plugins
        .add_plugins((
            MinimalPlugins,
            LogPlugin {
                filter: "streamdeck-cli=trace,bevy_streamdeck=trace".into(),
                level: log::Level::TRACE,
            },
            AssetPlugin::default(),
            ImagePlugin::default(),
        ))
        // StreamDeck Plugin
        .add_plugins(StreamDeckPlugin)
        // Load assets
        .insert_resource(ImageCatalog::default())
        .add_systems(Startup, load_assets)
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
    mut button_image: ResMut<streamdeck::ButtonImage>,
) {
    info!("Setting pumpkin image");
    if let Some(pumpkin_handle) = image_catalog.handles.get("pumpkin") {
        *button_image = streamdeck::ButtonImage(6, pumpkin_handle.clone());
    }
}

fn change_backlight_level(
    mut brightness: ResMut<streamdeck::Brightness>,
    // encoders: Res<Axis<streamdeck::Encoder>>,
) {
    info!("Updating brightness");
    *brightness = streamdeck::Brightness(100);

    // if encoders.is_changed() {
    //     let new_brightness = encoders
    //         .get_unclamped(streamdeck::Encoder(0))
    //         .unwrap_or(100.0)
    //         .clamp(0.0, 100.0);
    //
    //     info!("New brightness: {}", new_brightness);
    //
    //     *brightness = streamdeck::Brightness(new_brightness as u8);
    // }
}
