use std::rc::Rc;

use anyhow::Result;
use bevy::log::tracing_subscriber::registry;
use bevy::{app::AppExit, prelude::*, tasks::IoTaskPool};
use crossbeam_channel::unbounded;
use pipewire::node::Node;
use pipewire::types::ObjectType;
use pipewire::{context::Context, main_loop::MainLoop};
use tracing::debug;

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PreStartup, pipewire_init)
            .add_systems(PreUpdate, pipewire_events);
    }
}

#[derive(Debug)]
enum SoundAction {
    Shutdown,
}

#[derive(Resource)]
struct SoundActionSender(pipewire::channel::Sender<SoundAction>);

#[derive(Debug)]
enum SoundEvent {}

#[derive(Debug, Resource)]
struct SoundEventListener(crossbeam_channel::Receiver<SoundEvent>);

fn pipewire_events(events_listener: Res<SoundEventListener>) {
    for event in events_listener.0.try_iter() {
        trace!("Got SoundEvent: {:?}", event);
    }
}

fn pipewire_init(mut commands: Commands) {
    let (events_tx, events_rx) = unbounded::<SoundEvent>();
    let (actions_tx, actions_rx) = pipewire::channel::channel::<SoundAction>();

    let taskpool = IoTaskPool::get();
    taskpool
        .spawn(pipewire_thread(events_tx, actions_rx))
        .detach();

    commands.insert_resource(SoundEventListener(events_rx));
    commands.insert_resource(SoundActionSender(actions_tx));
}

async fn pipewire_thread(
    events_tx: crossbeam_channel::Sender<SoundEvent>,
    actions_rx: pipewire::channel::Receiver<SoundAction>,
) -> Result<()> {
    debug!("Pipewire Thread Started");

    // Setup our mainloop
    let mainloop = MainLoop::new(None)?;
    let context = Context::new(&mainloop)?;
    let core = context.connect(None)?;
    let registry = Rc::new(core.get_registry()?);
    let registry_weak = Rc::downgrade(&registry);

    // Attach our actions channel to the mainloop
    let _reciever = actions_rx.attach(mainloop.loop_(), |action| {
        trace!("Recieved SoundAction: {:?}", action);
    });

    // Listen for events
    let _listener = registry
        .add_listener_local()
        .global(move |global| {
            if let Some(registry) = registry_weak.upgrade() {
                if global.type_ == ObjectType::Node {
                    trace!("Received event: {:#?}", global);
                    let props = global.props.as_ref().unwrap();
                    dbg!(props);

                    let node: Node = registry.bind(global).unwrap();
                    dbg!(node);
                }
            }
        })
        .register();

    trace!("Starting mainloop");
    mainloop.run();

    debug!("Pipewire Thread Ending");
    Ok(())
}

fn old_pipewire_init() -> Result<()> {
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
