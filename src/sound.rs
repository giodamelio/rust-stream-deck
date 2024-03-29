mod pulse;

use anyhow::Result;
use bevy::{prelude::*, tasks::IoTaskPool};
use crossbeam_channel::unbounded;
use pulseaudio::protocol::{self, Command, SubscriptionEvent};

use crate::sound::pulse::PulseAudioSession;

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PreStartup, pulseaudio_init)
            .add_systems(PreUpdate, pipewire_events);
    }
}

#[derive(Resource)]
struct SoundActionSender(crossbeam_channel::Sender<Command>);

#[derive(Debug)]
enum SoundEvent {}

#[derive(Debug, Resource)]
struct SoundEventListener(crossbeam_channel::Receiver<SubscriptionEvent>);

#[derive(Debug, Component)]
pub struct Output;

#[derive(Debug, Component)]
pub struct Name(String);

#[derive(Debug, Component)]
pub struct Index(usize);

#[derive(Debug, Component)]
pub struct Volume(f32);

#[derive(Debug, Component)]
pub struct Muted(bool);

fn pipewire_events(events_listener: Res<SoundEventListener>) {
    for event in events_listener.0.try_iter() {
        trace!("Got SoundEvent: {:?}", event);
    }
}

fn pulseaudio_init(mut commands: Commands) {
    let (events_tx, events_rx) = unbounded::<SubscriptionEvent>();
    let (actions_tx, actions_rx) = unbounded::<Command>();

    commands.spawn((
        Output,
        Index(179),
        Name("Spotify".to_string()),
        Muted(false),
        Volume(0.5),
    ));

    let taskpool = IoTaskPool::get();

    // Subscribe and forward pulseaudio events
    taskpool
        .spawn(pulseaudio_subscription_thread(events_tx))
        .detach();

    // Perform pulseaudio commands
    taskpool
        .spawn(pulseaudio_command_thread(actions_rx))
        .detach();

    commands.insert_resource(SoundEventListener(events_rx));
    commands.insert_resource(SoundActionSender(actions_tx));
}

async fn pulseaudio_subscription_thread(
    events_tx: crossbeam_channel::Sender<SubscriptionEvent>,
) -> Result<()> {
    let name = format!("{}-{}", env!("CARGO_PKG_NAME"), "-subscriber");
    let mut pa = PulseAudioSession::new(name)?;
    pa.subscribe(protocol::SubscriptionMask::ALL, &events_tx)?;

    Ok(())
}

async fn pulseaudio_command_thread(
    _actions_rx: crossbeam_channel::Receiver<Command>,
) -> Result<()> {
    let name = format!("{}-{}", env!("CARGO_PKG_NAME"), "-action");
    let mut pa = PulseAudioSession::new(name)?;

    Ok(())
}
