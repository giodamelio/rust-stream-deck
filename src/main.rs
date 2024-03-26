mod streamdeck;

use bevy::{
    app::{App, Update},
    asset::AssetPlugin,
    log::LogPlugin,
    render::texture::ImagePlugin,
    MinimalPlugins,
};

use crate::streamdeck::StreamDeckPlugin;

fn main() -> anyhow::Result<()> {
    App::new()
        .add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default(),
            LogPlugin::default(),
        ))
        .add_plugins(StreamDeckPlugin)
        .add_systems(Update, log)
        .run();

    Ok(())
}

fn log() {
    // println!("HELLO BEVY");
}
