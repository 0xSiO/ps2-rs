// For details, see https://web.archive.org/web/20200616182207/https://www.win.tue.nl/~aeb/linux/kbd/scancodes-10.html#ss10.3
pub enum KeyboardType {
    XT,
    ATWithTranslation,
    MF2,
    MF2WithTranslation,
    ThinkPad,
    ThinkPadWithTranslation,
    Unknown122Key,
    IBM1390876,
    NetworkComputingDevicesN97,
    NetworkComputingDevicesSunLayout,
    OldJapaneseG,
    OldJapaneseP,
    OldJapaneseA,
    Unknown(u8, u8),
}

impl From<(u8, u8)> for KeyboardType {
    fn from(pair: (u8, u8)) -> Self {
        match pair {
            (0xab, 0x83) => KeyboardType::MF2,
            (0xab, 0x41) | (0xab, 0xc1) => KeyboardType::MF2WithTranslation,
            (0xab, 0x84) => KeyboardType::ThinkPad,
            (0xab, 0x54) => KeyboardType::ThinkPadWithTranslation,
            (0xab, 0x86) => KeyboardType::Unknown122Key,
            (0xbf, 0xbf) => KeyboardType::IBM1390876,
            (0xab, 0x85) => KeyboardType::NetworkComputingDevicesN97,
            (0xac, 0xa1) => KeyboardType::NetworkComputingDevicesSunLayout,
            (0xab, 0x90) => KeyboardType::OldJapaneseG,
            (0xab, 0x91) => KeyboardType::OldJapaneseP,
            (0xab, 0x92) => KeyboardType::OldJapaneseA,
            (first, second) => KeyboardType::Unknown(first, second),
        }
    }
}
