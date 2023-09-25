use bevy::prelude::*;
use image::{DynamicImage, GenericImageView, Rgb};
use std::fmt::{Debug, Formatter};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Button(pub u8);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Encoder(pub u8);

#[derive(Event, Debug, Clone, Copy)]
pub struct ButtonInput {
    pub button: Button,
    pub state: ButtonState,
}

impl ButtonInput {
    pub fn new(button: Button, state: ButtonState) -> Self {
        ButtonInput { button, state }
    }

    pub fn index(self) -> u8 {
        self.button.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Pressed,
    Released,
}

#[derive(Event, Clone)]
pub enum Command {
    Shutdown,
    SetBrightness(u8),
    SetButtonImage(u8, DynamicImage),
    SetButtonColor(u8, Rgb<u8>),
}

impl Debug for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Shutdown => {
                write!(f, "Shutdown")
            }
            Command::SetBrightness(level) => {
                write!(f, "SetBrightness(level = {})", level)
            }
            Command::SetButtonImage(index, data) => {
                write!(
                    f,
                    "SetButtonImage(button_index = {}, image_size = {:?})",
                    index,
                    data.dimensions()
                )
            }
            Command::SetButtonColor(index, color) => {
                write!(
                    f,
                    "SetButtonColor(button_index = {}, color = {:?})",
                    index, color
                )
            }
        }
    }
}
