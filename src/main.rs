mod streamdeck;

use bevy::{
    asset::AssetPlugin,
    prelude::{App, ButtonInput, Res, ResMut, Startup, Update},
    render::texture::ImagePlugin,
    MinimalPlugins,
};

use crate::streamdeck::{
    EncoderPosition, StreamDeck, StreamDeckButton, StreamDeckEncoder, StreamDeckPlugin,
};

fn main() -> anyhow::Result<()> {
    pretty_env_logger::try_init()?;

    App::new()
        .add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default(),
        ))
        .add_plugins(StreamDeckPlugin)
        .add_systems(Startup, set_brightness)
        .add_systems(Update, random_colors)
        .run();

    Ok(())
}

fn random_colors(mut deck: ResMut<StreamDeck>, button: Res<ButtonInput<StreamDeckButton>>) {
    let b = StreamDeckButton(0);
    if button.just_pressed(b) {
        log::info!("Random Color Incoming");
        let color = image::Rgb::<u8>([rand::random(), rand::random(), rand::random()]);
        let _ = deck.button_set_color(b, color);
    }

    if button.just_pressed(StreamDeckButton(1)) {
        log::info!("Setting brightness to max");
        let _ = deck.set_backlight(100);
    }
}

fn set_brightness(mut deck: ResMut<StreamDeck>) {
    log::info!("Setting brightness to max");
    let _ = deck.set_backlight(100);
}
