use crate::error::{StreamDeckError, StreamDeckResult};
use hidapi::DeviceInfo;
use std::marker::PhantomData;

use crate::device_constants::*;

#[derive(Clone, Debug)]
pub(crate) struct Grid<T> {
    height: usize,
    width: usize,
    #[doc(hidden)]
    _t: PhantomData<T>,
}

impl<T> Grid<T> {
    fn count(self) -> usize {
        self.width * self.height
    }

    const fn new(height: usize, width: usize) -> Option<Self> {
        Some(Self {
            height,
            width,
            _t: PhantomData,
        })
    }
}

#[derive(Clone, Debug)]
struct Button;
#[derive(Clone, Debug)]
struct Knob;
#[derive(Clone, Debug)]
struct Peddle;

#[derive(Clone)]
pub enum Kind {
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

#[derive(Clone, Debug)]
pub struct Model {
    pub kind: Kind,
    vendor_id: u16,
    product_ud: u16,
    button_grid: Option<Grid<Button>>,
    knob_grid: Option<Grid<Knob>>,
    peddle_grid: Option<Grid<Peddle>>,
}

impl Model {
    pub const ORIGINAL: Self = Self {
        kind: Kind::Original,
        vendor_id: VENDOR_ID,
        product_ud: PRODUCT_ID_ORIGINAL,
        button_grid: Grid::new(3, 5),
        knob_grid: None,
        peddle_grid: None,
    };

    pub const ORIGINAL_V2: Self = Self {
        kind: Kind::OriginalV2,
        vendor_id: VENDOR_ID,
        product_ud: PRODUCT_ID_ORIGINAL_V2,
        button_grid: Grid::new(3, 5),
        knob_grid: None,
        peddle_grid: None,
    };

    pub const MINI: Self = Self {
        kind: Kind::Mini,
        vendor_id: VENDOR_ID,
        product_ud: PRODUCT_ID_MINI,
        button_grid: Grid::new(2, 3),
        knob_grid: None,
        peddle_grid: None,
    };

    pub const XL: Self = Self {
        kind: Kind::XL,
        vendor_id: VENDOR_ID,
        product_ud: PRODUCT_ID_XL,
        button_grid: Grid::new(4, 8),
        knob_grid: None,
        peddle_grid: None,
    };

    pub const XLV2: Self = Self {
        kind: Kind::XLV2,
        vendor_id: VENDOR_ID,
        product_ud: PRODUCT_ID_XL_V2,
        button_grid: Grid::new(4, 8),
        knob_grid: None,
        peddle_grid: None,
    };

    pub const MK2: Self = Self {
        kind: Kind::MK2,
        vendor_id: VENDOR_ID,
        product_ud: PRODUCT_ID_MK2,
        button_grid: Grid::new(3, 5),
        knob_grid: None,
        peddle_grid: None,
    };

    pub const MINI_MK2: Self = Self {
        kind: Kind::MiniMK2,
        vendor_id: VENDOR_ID,
        product_ud: PRODUCT_ID_MINI_MK2,
        button_grid: Grid::new(2, 3),
        knob_grid: None,
        peddle_grid: None,
    };

    pub const PEDAL: Self = Self {
        kind: Kind::Pedal,
        vendor_id: VENDOR_ID,
        product_ud: PRODUCT_ID_PEDAL,
        button_grid: None,
        knob_grid: None,
        peddle_grid: Grid::new(1, 3),
    };

    pub const PLUS: Self = Self {
        kind: Kind::Plus,
        vendor_id: VENDOR_ID,
        product_ud: PRODUCT_ID_PLUS,
        button_grid: Grid::new(2, 4),
        knob_grid: Grid::new(1, 4),
        peddle_grid: None,
    };
}

impl Model {
    pub fn from_device_info(info: &DeviceInfo) -> StreamDeckResult<Self> {
        let vendor_id = info.vendor_id();
        if vendor_id != VENDOR_ID {
            return Err(StreamDeckError::InvalidVendorID(vendor_id));
        }

        Self::from_product_id(info.product_id())
    }

    pub fn from_product_id(product_id: u16) -> StreamDeckResult<Self> {
        match product_id {
            PRODUCT_ID_ORIGINAL => Ok(Self::ORIGINAL),
            PRODUCT_ID_ORIGINAL_V2 => Ok(Self::ORIGINAL_V2),
            PRODUCT_ID_MINI => Ok(Self::MINI),
            PRODUCT_ID_XL => Ok(Self::XL),
            PRODUCT_ID_XL_V2 => Ok(Self::XLV2),
            PRODUCT_ID_MK2 => Ok(Self::MK2),
            PRODUCT_ID_MINI_MK2 => Ok(Self::MINI_MK2),
            PRODUCT_ID_PEDAL => Ok(Self::PEDAL),
            PRODUCT_ID_PLUS => Ok(Self::PLUS),
            id => Err(StreamDeckError::InvalidProductID(id)),
        }
    }
}

impl From<Kind> for Model {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::Original => Model::ORIGINAL,
            Kind::OriginalV2 => Model::ORIGINAL_V2,
            Kind::Mini => Model::MINI,
            Kind::XL => Model::XL,
            Kind::XLV2 => Model::XLV2,
            Kind::MK2 => Model::MK2,
            Kind::MiniMK2 => Model::MINI_MK2,
            Kind::Pedal => Model::PEDAL,
            Kind::Plus => Model::PLUS,
        }
    }
}

impl std::fmt::Debug for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Kind::Original => "Original",
            Kind::OriginalV2 => "Original v2",
            Kind::Mini => "Mini",
            Kind::XL => "XL",
            Kind::XLV2 => "XL v2",
            Kind::MK2 => "MK2",
            Kind::MiniMK2 => "Mini MK2",
            Kind::Pedal => "Pedal",
            Kind::Plus => "Plus",
        };
        write!(f, "{}", name)
    }
}

impl std::fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind.to_string())
    }
}
