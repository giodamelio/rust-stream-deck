use std::{ffi::CString, os::unix::net::UnixStream};

use anyhow::{anyhow, Result};
use bevy::{prelude::*, tasks::IoTaskPool};
use crossbeam_channel::unbounded;
use pulseaudio::protocol::{
    self, read_ack_message, ChannelVolume, Command, SetStreamMuteParams, SetStreamVolumeParams,
};

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PreStartup, pipewire_init)
            .add_systems(PreUpdate, pipewire_events);
    }
}

#[derive(Resource)]
struct SoundActionSender(crossbeam_channel::Sender<Command>);

#[derive(Debug)]
enum SoundEvent {}

#[derive(Debug, Resource)]
struct SoundEventListener(crossbeam_channel::Receiver<SoundEvent>);

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

fn pipewire_init(mut commands: Commands) {
    let (events_tx, events_rx) = unbounded::<SoundEvent>();
    let (actions_tx, actions_rx) = unbounded::<Command>();

    commands.spawn((
        Output,
        Index(179),
        Name("Spotify".to_string()),
        Muted(false),
        Volume(0.5),
    ));

    let taskpool = IoTaskPool::get();
    taskpool
        .spawn(pipewire_thread(events_tx, actions_rx))
        .detach();

    commands.insert_resource(SoundEventListener(events_rx));
    commands.insert_resource(SoundActionSender(actions_tx));
}

async fn pipewire_thread(
    _events_tx: crossbeam_channel::Sender<SoundEvent>,
    actions_rx: crossbeam_channel::Receiver<Command>,
) -> Result<()> {
    let mut sequence = 0;

    // Find and connect to PulseAudio. The socket is usually in a well-known
    // location under XDG_RUNTIME_DIR.
    let socket_path =
        pulseaudio::socket_path_from_env().ok_or(anyhow!("PulseAudio not available"))?;
    let mut sock = std::io::BufReader::new(UnixStream::connect(socket_path)?);

    // PulseAudio usually puts an authentication "cookie" in ~/.config/pulse/cookie.
    let cookie = pulseaudio::cookie_path_from_env()
        .and_then(|path| std::fs::read(path).ok())
        .unwrap_or_default();
    let auth = protocol::AuthParams {
        version: protocol::MAX_VERSION,
        supports_shm: false,
        supports_memfd: false,
        cookie,
    };

    // Write the auth "command" to the socket, and read the reply. The reply
    // contains the negotiated protocol version.
    protocol::write_command_message(
        sock.get_mut(),
        sequence,
        protocol::Command::Auth(auth),
        protocol::MAX_VERSION,
    )?;
    let (_, auth_info) =
        protocol::read_reply_message::<protocol::AuthReply>(&mut sock, protocol::MAX_VERSION)?;
    let protocol_version = std::cmp::min(protocol::MAX_VERSION, auth_info.version);
    sequence += 1;

    // The next step is to set the client name.
    let mut props = protocol::Props::new();
    props.set(
        protocol::Prop::ApplicationName,
        CString::new(env!("CARGO_PKG_NAME")).unwrap(),
    );
    protocol::write_command_message(
        sock.get_mut(),
        sequence,
        protocol::Command::SetClientName(props),
        protocol_version,
    )?;

    let _ =
        protocol::read_reply_message::<protocol::SetClientNameReply>(&mut sock, protocol_version)?;
    sequence += 1;

    // Perform commands forever
    loop {
        let cmd = actions_rx.recv()?;

        match cmd {
            Command::SetSinkInputMute(_params) => {
                protocol::write_command_message(sock.get_mut(), 3, cmd, protocol_version)?;
                let _resp = read_ack_message(&mut sock)?;
            }
            cmd => {
                trace!("Unknown command: {:?}", cmd)
            }
        }

        sequence += 1;
    }
}
