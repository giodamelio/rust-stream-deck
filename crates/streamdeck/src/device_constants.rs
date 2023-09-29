use binrw::prelude::*;
use binrw::NullString;

pub const VENDOR_ID: u16 = 0x0FD9;

pub const PRODUCT_ID_ORIGINAL: u16 = 0x0060;
pub const PRODUCT_ID_ORIGINAL_V2: u16 = 0x006D;
pub const PRODUCT_ID_MINI: u16 = 0x0063;
pub const PRODUCT_ID_XL: u16 = 0x006c;
pub const PRODUCT_ID_XL_V2: u16 = 0x008F;
pub const PRODUCT_ID_MK2: u16 = 0x0080;
pub const PRODUCT_ID_MINI_MK2: u16 = 0x0090;
pub const PRODUCT_ID_PEDAL: u16 = 0x0086;
pub const PRODUCT_ID_PLUS: u16 = 0x0084;

pub const PRODUCT_ID_ALL: [u16; 9] = [
    PRODUCT_ID_ORIGINAL,
    PRODUCT_ID_ORIGINAL_V2,
    PRODUCT_ID_MINI,
    PRODUCT_ID_XL,
    PRODUCT_ID_XL_V2,
    PRODUCT_ID_MK2,
    PRODUCT_ID_MINI_MK2,
    PRODUCT_ID_PEDAL,
    PRODUCT_ID_PLUS,
];

// Feature Reports
#[derive(Debug)]
#[binrw]
pub struct FeatureReportFirmwareVersion {
    feature_id: u8,
    length: u8,
    // Remove 4 junk bytes
    #[br(pad_before = 4)]
    pub version: NullString,
}

#[derive(Debug)]
#[binrw]
pub struct FeatureReportSerialNumber {
    feature_id: u8,
    length: u8,
    pub serial: NullString,
}
