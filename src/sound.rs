use anyhow::Result;
use bevy::{prelude::*, tasks::IoTaskPool};
use crossbeam_channel::unbounded;

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
struct SoundActionSender(crossbeam_channel::Sender<SoundAction>);

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
    let (actions_tx, actions_rx) = unbounded::<SoundAction>();

    let taskpool = IoTaskPool::get();
    taskpool
        .spawn(pipewire_thread(events_tx, actions_rx))
        .detach();

    commands.insert_resource(SoundEventListener(events_rx));
    commands.insert_resource(SoundActionSender(actions_tx));
}

async fn pipewire_thread(
    _events_tx: crossbeam_channel::Sender<SoundEvent>,
    _actions_rx: crossbeam_channel::Receiver<SoundAction>,
) -> Result<()> {
    Ok(())
}
