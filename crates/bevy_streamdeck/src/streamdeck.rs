use std::fmt::{Debug, Formatter};

use bevy::input::ButtonState;
use bevy::prelude::*;
use image::{DynamicImage, GenericImageView, Rgb};

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

#[derive(Event, Clone)]
pub enum Command {
    Shutdown,
    SetBrightness(u8),
    SetButtonImage(u8, DynamicImage),
    SetButtonImageData(u8, Vec<u8>),
    SetButtonColor(u8, Rgb<u8>),
    LCDCenterText(String),
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
            Command::SetButtonImageData(index, data) => {
                write!(
                    f,
                    "SetButtonImageData(button_index = {}, data_length = {})",
                    index,
                    data.len()
                )
            }
            Command::SetButtonColor(index, color) => {
                write!(
                    f,
                    "SetButtonColor(button_index = {}, color = {:?})",
                    index, color
                )
            }
            Command::LCDCenterText(text) => {
                write!(f, "LCDCenterText(text = {})", text)
            }
        }
    }
}
