use core::convert::TryFrom;

use crate::{error::MouseError, mouse::Result};

#[repr(u8)]
pub enum MouseResolution {
    OneCountPerMM = 0x00,
    TwoCountPerMM = 0x01,
    FourCountPerMM = 0x02,
    EightCountPerMM = 0x03,
}

impl TryFrom<u8> for MouseResolution {
    type Error = MouseError;

    fn try_from(value: u8) -> Result<Self> {
        use MouseResolution::*;
        match value {
            0x00 => Ok(OneCountPerMM),
            0x01 => Ok(TwoCountPerMM),
            0x02 => Ok(FourCountPerMM),
            0x03 => Ok(EightCountPerMM),
            other => Err(MouseError::InvalidResolution(other)),
        }
    }
}
