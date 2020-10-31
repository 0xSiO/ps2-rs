#[derive(Debug, PartialEq, Eq)]
pub enum MouseType {
    StandardPS2,
    PS2WithScrollWheel,
    FiveButton,
    Unknown(u8),
}

impl From<u8> for MouseType {
    fn from(value: u8) -> Self {
        match value {
            0x00 => MouseType::StandardPS2,
            0x03 => MouseType::PS2WithScrollWheel,
            0x04 => MouseType::FiveButton,
            other => MouseType::Unknown(other),
        }
    }
}
