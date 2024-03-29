use std::{ffi::CString, os::unix::net::UnixStream};

use anyhow::{anyhow, bail, ensure, Result};
use bevy::prelude::*;
use pulseaudio::protocol;

#[derive(Debug)]
pub struct PulseAudioSession {
    sequence: u32,
    socket: std::io::BufReader<UnixStream>,
    protocol_version: u16,
}

impl PulseAudioSession {
    pub fn new(client_name: String) -> Result<Self> {
        // Connect to the PulseAudio socket
        let socket_path =
            pulseaudio::socket_path_from_env().ok_or(anyhow!("PulseAudio not available"))?;
        let mut sock = std::io::BufReader::new(UnixStream::connect(socket_path)?);

        // Find the Auth Cookie
        let cookie = pulseaudio::cookie_path_from_env()
            .and_then(|path| std::fs::read(path).ok())
            .unwrap_or_default();
        let auth = protocol::AuthParams {
            version: protocol::MAX_VERSION,
            supports_shm: false,
            supports_memfd: false,
            cookie,
        };

        // Authenticate with the socket
        protocol::write_command_message(
            sock.get_mut(),
            0,
            protocol::Command::Auth(auth),
            protocol::MAX_VERSION,
        )?;
        let (seq, auth_info) =
            protocol::read_reply_message::<protocol::AuthReply>(&mut sock, protocol::MAX_VERSION)?;
        ensure!(seq == 0, "Sequence Mismatch");
        let protocol_version = std::cmp::min(protocol::MAX_VERSION, auth_info.version);

        // The next step is to set the client name.
        let mut props = protocol::Props::new();
        props.set(
            protocol::Prop::ApplicationName,
            CString::new(client_name).unwrap(),
        );
        protocol::write_command_message(
            sock.get_mut(),
            1,
            protocol::Command::SetClientName(props),
            protocol_version,
        )?;

        // Read the client name set reply
        let (seq, _) = protocol::read_reply_message::<protocol::SetClientNameReply>(
            &mut sock,
            protocol_version,
        )?;
        ensure!(seq == 1, "Sequence Mismatch");

        Ok(Self {
            sequence: 2,
            socket: sock,
            protocol_version,
        })
    }

    pub fn get_sink_inputs(&mut self) -> Result<Vec<protocol::SinkInputInfo>> {
        // Send Command
        protocol::write_command_message(
            self.socket.get_mut(),
            self.sequence,
            protocol::Command::GetSinkInputInfoList,
            self.protocol_version,
        )?;

        // Read Response
        let (ack_seq, info_list) = protocol::read_reply_message::<protocol::SinkInputInfoList>(
            &mut self.socket,
            self.protocol_version,
        )?;
        ensure!(self.sequence == ack_seq, anyhow!("Sequence Mismatch"));
        self.sequence += 1;

        Ok(info_list)
    }

    pub fn write(&mut self, command: protocol::Command) -> Result<()> {
        match command {
            protocol::Command::Subscribe(_) => {
                // Send Command
                protocol::write_command_message(
                    self.socket.get_mut(),
                    self.sequence,
                    command,
                    self.protocol_version,
                )?;

                // Get ACK
                let ack_seq = protocol::read_ack_message(&mut self.socket)?;
                ensure!(self.sequence == ack_seq, anyhow!("Sequence Mismatch"));
                self.sequence += 1;

                Ok(())
            }
            unknown => {
                debug!("Unknown command type: {:?}", unknown);

                Ok(())
            }
        }
    }

    pub fn subscribe(
        &mut self,
        mask: protocol::SubscriptionMask,
        sender: &crossbeam_channel::Sender<protocol::SubscriptionEvent>,
    ) -> Result<()> {
        self.write(protocol::Command::Subscribe(mask))?;

        loop {
            let (_, event) =
                protocol::read_command_message(&mut self.socket, self.protocol_version)?;

            match event {
                protocol::Command::SubscribeEvent(event) => {
                    if let Err(err) = sender.send(event) {
                        error!("Could not send event: {:?}", err);
                    }
                }
                _ => error!("Got unexpected event {:?}", event),
            }
        }
    }
}
