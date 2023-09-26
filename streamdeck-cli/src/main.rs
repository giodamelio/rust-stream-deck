use bevy::asset::LoadState;
use bevy::input::ButtonState;
use bevy::log::{self, LogPlugin};
use bevy::prelude::*;
use rand::Rng;
use std::collections::HashMap;

use bevy_streamdeck::{streamdeck, StreamDeckKind, StreamDeckPlugin};

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
        // Keep track of which app we are using
        .add_state::<AppState>()
        // Load assets
        .insert_resource(ImageCatalog::default())
        .add_systems(Startup, load_assets)
        .add_systems(Startup, spawn_apps)
        // Change the backlight level with a knob
        .add_systems(Update, change_backlight_level)
        // Set a button image on the streamdeck
        .add_systems(Update, set_button_random_color)
        // Set the systems for our states
        .add_systems(OnEnter(AppState::Welcome), welcome_setup)
        .add_systems(Update, (welcome_update).run_if(in_state(AppState::Welcome)))
        .run();
}

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
enum AppState {
    #[default]
    Welcome,
    Settings,
    AppSelector,
    App,
}

// Make all the keys grey and say welcome
fn welcome_setup(mut ev_command: EventWriter<streamdeck::Command>, deck_kind: Res<StreamDeckKind>) {
    let color = [44, 44, 44];

    info!("Screen Size: {:?}", deck_kind.lcd_strip_size());
    for i in 0..deck_kind.key_count() {
        ev_command.send(streamdeck::Command::SetButtonColor(i, color.into()));
    }
    ev_command.send(streamdeck::Command::LCDCenterText("YAY".to_string()));

    info!("WELCOME");
}

fn welcome_update(
    mut ev_button: EventReader<streamdeck::ButtonInput>,
    // mut ev_command: EventWriter<streamdeck::Command>,
) {
    for event in ev_button.iter() {
        if event.state != ButtonState::Released {
            continue;
        }

        info!("Button Pressed!");
    }
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

            if event.state != ButtonState::Released {
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

        ev_command.send(streamdeck::Command::SetBrightness(new_brightness as u8))
    }
}
