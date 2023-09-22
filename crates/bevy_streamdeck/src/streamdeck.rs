use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Button(pub u8);

#[derive(Resource, Default, Debug, Clone, PartialEq)]
pub struct ButtonImage(pub u8, pub Handle<Image>);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Encoder(pub u8);

#[derive(Resource, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Brightness(pub u8);

impl Default for Brightness {
    fn default() -> Self {
        Self(100)
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    SetBrightness(u8),
    SetButtonImage(u8, Handle<Image>),
}
