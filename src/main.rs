mod streamdeck;

use std::time::Duration;

use bevy::{
    asset::AssetPlugin,
    prelude::{App, ButtonInput, IntoSystemConfigs, Res, Update},
    render::texture::ImagePlugin,
    time::common_conditions::on_timer,
    MinimalPlugins,
};

use crate::streamdeck::{EncoderPosition, StreamDeckButton, StreamDeckEncoder, StreamDeckPlugin};

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
        .add_systems(
            Update,
            log_encoders.run_if(on_timer(Duration::from_secs(1))),
        )
        .run();

    Ok(())
}

fn log_buttons(
    button: Res<ButtonInput<StreamDeckButton>>,
    encoder: Res<ButtonInput<StreamDeckEncoder>>,
) {
    if button.just_pressed(StreamDeckButton(0)) {
        log::info!("Button 0 pressed");
    }

    if encoder.just_pressed(StreamDeckEncoder(0)) {
        log::info!("Encoder 0 pressed");
    }
}

fn log_encoders(knob: Res<EncoderPosition>) {
    log::info!("Knob: {:?}", knob.get_position(StreamDeckEncoder(0)));
    log::info!(
        "Knob Clamped: {:?}",
        knob.get_position_clamped(StreamDeckEncoder(0), 0, 100)
    );
}
