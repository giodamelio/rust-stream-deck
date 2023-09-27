use crate::error::{StreamDeckError, StreamDeckResult};
use hidapi::DeviceInfo;
use std::fmt::{Debug, Display, Formatter};

use crate::device_constants::*;

#[derive(Clone)]
pub enum Model {
    Original,
    OriginalV2,
    Mini,
    XL,
    XLV2,
    MK2,
    MiniMK2,
    Pedal,
    Plus,
}

impl Model {
    pub fn from_device_info(info: &DeviceInfo) -> StreamDeckResult<Self> {
        let vendor_id = info.vendor_id();
        if vendor_id != VENDOR_ID {
            return Err(StreamDeckError::InvalidVendorID(vendor_id));
        }

        Model::from_product_id(info.product_id())
    }

    pub fn from_product_id(product_id: u16) -> StreamDeckResult<Self> {
        match product_id {
            PRODUCT_ID_ORIGINAL => Ok(Self::Original),
            PRODUCT_ID_ORIGINAL_V2 => Ok(Self::OriginalV2),
            PRODUCT_ID_MINI => Ok(Self::Mini),
            PRODUCT_ID_XL => Ok(Self::XL),
            PRODUCT_ID_XL_V2 => Ok(Self::XLV2),
            PRODUCT_ID_MK2 => Ok(Self::MK2),
            PRODUCT_ID_MINI_MK2 => Ok(Self::MiniMK2),
            PRODUCT_ID_PEDAL => Ok(Self::Pedal),
            PRODUCT_ID_PLUS => Ok(Self::Plus),
            id => Err(StreamDeckError::InvalidProductID(id)),
        }
    }
}

impl Debug for Model {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl Display for Model {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Model::Original => "Original",
            Model::OriginalV2 => "Original v2",
            Model::Mini => "Mini",
            Model::XL => "XL",
            Model::XLV2 => "XL v2",
            Model::MK2 => "MK2",
            Model::MiniMK2 => "Mini MK2",
            Model::Pedal => "Pedal",
            Model::Plus => "Plus",
        };
        write!(f, "{}", name)
    }
}
