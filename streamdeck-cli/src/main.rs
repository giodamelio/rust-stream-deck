use bevy::asset::LoadState;
use bevy::log::{self, LogPlugin};
use bevy::prelude::*;
use rand::Rng;
use std::collections::HashMap;

use bevy_streamdeck::{streamdeck, StreamDeckPlugin};

fn main() {
    App::new()
        // Add some plugins
        .add_plugins((
            MinimalPlugins,
            LogPlugin {
                filter: "streamdeck-cli=trace,bevy_streamdeck=debug".into(),
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
        .add_systems(Startup, spawn_apps)
        // Change the backlight level with a knob
        .add_systems(Update, change_backlight_level)
        // Set a button image on the streamdeck
        .add_systems(Update, set_button_random_color)
        .run();
}

#[derive(Component)]
struct Enabled(bool);

#[derive(Bundle)]
struct DeckApp {
    enabled: Enabled,
}

fn spawn_apps(mut commands: Commands) {
    commands.spawn(DeckApp {
        enabled: Enabled(true),
    });
}

#[derive(Resource, Default)]
struct ImageCatalog {
    handles: HashMap<&'static str, Handle<Image>>,
}

fn load_assets(asset_server: Res<AssetServer>, mut image_catalog: ResMut<ImageCatalog>) {
    let pumpkin_handle: Handle<Image> = asset_server.load("pumpkin.png");
    image_catalog.handles.insert("pumpkin", pumpkin_handle);
}

fn set_button_random_color(
    mut ev_button: EventReader<streamdeck::ButtonInput>,
    mut ev_command: EventWriter<streamdeck::Command>,
    image_catalog: Res<ImageCatalog>,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
) {
    for event in ev_button.iter() {
        if let Some(pumpkin_handle) = image_catalog.handles.get("pumpkin") {
            // Make sure the asset is loaded
            if asset_server.get_load_state(pumpkin_handle) != LoadState::Loaded {
                return;
            }

            if event.state != streamdeck::ButtonState::Released {
                return;
            }

            let mut rng = rand::thread_rng();
            let mut color = [0u8; 3];
            rng.fill(&mut color);

            ev_command.send(streamdeck::Command::SetButtonColor(
                event.index(),
                color.into(),
            ));
        }
    }
}

fn change_backlight_level(
    encoders: Res<Axis<streamdeck::Encoder>>,
    mut ev_command: EventWriter<streamdeck::Command>,
) {
    if encoders.is_changed() {
        let new_brightness = encoders
            .get_unclamped(streamdeck::Encoder(0))
            .unwrap_or(100.0)
            .clamp(0.0, 100.0);

        info!("New brightness: {}", new_brightness);
        ev_command.send(streamdeck::Command::SetBrightness(new_brightness as u8))
    }
}
