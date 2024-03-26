mod streamdeck;

use bevy::{
    asset::AssetPlugin,
    prelude::{App, ButtonInput, Res, Update},
    render::texture::ImagePlugin,
    MinimalPlugins,
};

use crate::streamdeck::{StreamDeckButton, StreamDeckPlugin};

fn main() -> anyhow::Result<()> {
    pretty_env_logger::try_init()?;

    App::new()
        .add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default(),
        ))
        .add_plugins(StreamDeckPlugin)
        .add_systems(Update, log_buttons)
        .run();

    Ok(())
}

fn log_buttons(button: Res<ButtonInput<StreamDeckButton>>) {
    if button.just_pressed(StreamDeckButton::Button(0)) {
        log::info!("Button 0 pressed");
    }

    if button.just_pressed(StreamDeckButton::Encoder(0)) {
        log::info!("Encoder 0 pressed");
    }
}
