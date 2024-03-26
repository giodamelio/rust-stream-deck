use bevy::{
    app::{App, Update},
    asset::AssetPlugin,
    log::LogPlugin,
    render::texture::ImagePlugin,
    MinimalPlugins,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default(),
            LogPlugin::default(),
        ))
        .add_systems(Update, log)
        .run();

    Ok(())
}

fn log() {
    println!("HELLO BEVY");
}
