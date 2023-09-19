use bevy::prelude::Resource;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct StreamDeckButton(pub u8);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct StreamDeckEncoder(pub u8);

#[derive(Resource, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct StreamDeckBrightness(pub u8);

impl Default for StreamDeckBrightness {
    fn default() -> Self {
        Self(100)
    }
}
