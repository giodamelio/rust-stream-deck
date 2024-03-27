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

use pipewire::types::ObjectType;
use pipewire::{context::Context, device::Device, main_loop::MainLoop};

fn main() -> anyhow::Result<()> {
    let mainloop = MainLoop::new(None)?;
    let context = Context::new(&mainloop)?;
    let core = context.connect(None)?;
    let registry = core.get_registry()?;

    let _listener = registry
        .add_listener_local()
        .global(|global| {
            if global.type_ == ObjectType::Port {
                let props = global.props.as_ref().unwrap();
                let port_name = props.get("port.name");
                let port_alias = props.get("port.alias");
                let object_path = props.get("object.path");
                let format_dsp = props.get("format.dsp");
                let audio_channel = props.get("audio.channel");
                let port_id = props.get("port.id");
                let port_direction = props.get("port.direction");
                println!("Port: Name: {:?}\n  Alias: {:?}\n  Id: {:?}\n  Direction: {:?}\n  AudioChannel: {:?}\n  Object Path: {:?}\n  FormatDsp: {:?}",
                    port_name,
                    port_alias,
                    port_id,port_direction,audio_channel,object_path,format_dsp
                );
            } else if global.type_ == ObjectType::Device {
                let props = global.props.as_ref().unwrap();
                let device_name = props.get("device.name");
                let device_nick = props.get("device.nick");
                let device_description = props.get("device.description");
                let device_api = props.get("device.api");
                let media_class = props.get("media.class");
                println!("Device: Name: {:?}\n  Nick: {:?}\n  Desc: {:?}\n  Api: {:?}\n  MediaClass: {:?}",
                    device_name, device_nick, device_description, device_api, media_class);
            }
        })
        .register();

    // Synchronize the registry and run the main loop
    mainloop.run();

    Ok(())
}

fn main2() -> anyhow::Result<()> {
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
