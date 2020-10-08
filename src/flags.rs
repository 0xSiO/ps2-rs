use bitflags::bitflags;

bitflags! {
    pub struct ControllerStatus: u8 {
        const OUTPUT_FULL        = 0b00000001;
        const INPUT_FULL         = 0b00000010;
        const SYSTEM_FLAG        = 0b00000100;
        const INPUT_IS_COMMAND   = 0b00001000;
        const KEYBOARD_LOCK      = 0b00010000;
        const MOUSE_OUTPUT_FULL  = 0b00100000;
        const TIMEOUT_ERR        = 0b01000000;
        const PARITY_ERR         = 0b10000000;
    }
}

bitflags! {
    pub struct ControllerConfig: u8 {
        const ENABLE_KEYBOARD_INTERRUPT = 0b00000001;
        const ENABLE_MOUSE_INTERRUPT    = 0b00000010;
        const SET_SYSTEM_FLAG           = 0b00000100;
        const DISABLE_KEYBOARD          = 0b00010000;
        const DISABLE_MOUSE             = 0b00100000;
        const ENABLE_TRANSLATE          = 0b01000000;
    }
}

bitflags! {
    pub struct ControllerInput: u8 {
        const KEYBOARD_DATA        = 0b00000001;
        const MOUSE_DATA           = 0b00000010;
        const ENABLE_EXTRA_RAM     = 0b00010000;
        const MANUFACTURING_JUMPER = 0b00100000;
        const MONOCHROME_DISPLAY   = 0b01000000;
        const KEYBOARD_ENABLED     = 0b10000000;
    }
}

bitflags! {
    pub struct ControllerOutput: u8 {
        const SYSTEM_RESET       = 0b00000001;
        const A20_GATE           = 0b00000010;
        const MOUSE_DATA         = 0b00000100;
        const MOUSE_CLOCK        = 0b00001000;
        const KEYBOARD_INTERRUPT = 0b00010000;
        const MOUSE_INTERRUPT    = 0b00100000;
        const KEYBOARD_CLOCK     = 0b01000000;
        const KEYBOARD_DATA      = 0b10000000;
    }
}

bitflags! {
    pub struct KeyboardLeds: u8 {
        const SCROLL_LOCK = 0b001;
        const NUM_LOCK    = 0b010;
        const CAPS_LOCK   = 0b100;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undefined_bits_test() {
        // Undefined bits in config byte and input port default to 0
        assert_eq!(ControllerConfig::all().bits(), 0b01110111);
        assert_eq!(ControllerInput::all().bits(), 0b11110011);
    }

    #[test]
    fn handles_all_zeroes_test() {
        assert_eq!(
            (
                ControllerConfig::from_bits_truncate(0).bits(),
                ControllerInput::from_bits_truncate(0).bits(),
                ControllerOutput::from_bits_truncate(0).bits()
            ),
            (0, 0, 0)
        );
    }
}
