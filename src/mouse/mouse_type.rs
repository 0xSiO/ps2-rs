/// PS/2 mouse device type. For more details, see [here](https://web.archive.org/web/20200616182210/https://www.win.tue.nl/%7Eaeb/linux/kbd/scancodes-13.html#ss13.3).
#[derive(Debug, PartialEq, Eq)]
pub enum MouseType {
    Standard,
    IntelliMouse,
    IntelliMouseExplorer,
    Typhoon,
    Unknown(u8),
}

impl From<u8> for MouseType {
    fn from(value: u8) -> Self {
        match value {
            0x00 => MouseType::Standard,
            0x03 => MouseType::IntelliMouse,
            0x04 => MouseType::IntelliMouseExplorer,
            0x08 => MouseType::Typhoon,
            other => MouseType::Unknown(other),
        }
    }
}
