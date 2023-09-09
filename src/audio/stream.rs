use crate::audio::stream::StreamState::Expired;
use windows::Win32::Media::Audio::AudioSessionState;

#[derive(Debug, Clone)]
pub struct Stream {
    pub friendly_name: Option<String>,
    pub state: StreamState,
    pub process_id: u32,
}

#[derive(Debug, Clone)]
pub enum StreamState {
    Inactive,
    Active,
    Expired,
}

impl From<AudioSessionState> for StreamState {
    fn from(state: AudioSessionState) -> Self {
        match state.0 {
            0 => Self::Inactive,
            1 => Self::Active,
            2 => Expired,
            s => unreachable!("Invalid AudioSessionState: {}", s),
        }
    }
}
