mod streamdeck;

use bevy::{
    app::{App, Update},
    asset::AssetPlugin,
    render::texture::ImagePlugin,
    MinimalPlugins,
};

use crate::streamdeck::StreamDeckPlugin;

fn main() -> anyhow::Result<()> {
    pretty_env_logger::try_init()?;

    App::new()
        .add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default(),
        ))
        .add_plugins(StreamDeckPlugin)
        .add_systems(Update, log)
        .run();

    Ok(())
}

fn log() {
    // info!("Hello Bevy!");
}
