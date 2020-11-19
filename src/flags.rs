use bitflags::bitflags;

bitflags! {
    pub struct ControllerStatus: u8 {
        /// Whether there is data available to read at port 0x60.
        const OUTPUT_FULL        = 0b00000001;
        /// Whether data has been written to port 0x60.
        const INPUT_FULL         = 0b00000010;
        /// Should be set if the system boots and the Basic Assurance Test passes successfully.
        /// Unset after a power-on reset.
        const SYSTEM_FLAG        = 0b00000100;
        /// Whether data written to port 0x60 is for a PS/2 controller command rather than a PS/2
        /// device.
        const INPUT_IS_COMMAND   = 0b00001000;
        /// Whether the keyboard functionality is inhibited.
        const KEYBOARD_LOCK      = 0b00010000;
        /// Whether there is data available to read from the mouse at port 0x60.
        const MOUSE_OUTPUT_FULL  = 0b00100000;
        /// Whether a timeout error occurred during command write or response.
        const TIMEOUT_ERR        = 0b01000000;
        /// Whether a communication error occurred.
        const PARITY_ERR         = 0b10000000;
    }
}

bitflags! {
    pub struct ControllerConfig: u8 {
        /// Whether the keyboard should trigger any interrupts.
        const ENABLE_KEYBOARD_INTERRUPT = 0b00000001;
        /// Whether the mouse should trigger any interrupts.
        const ENABLE_MOUSE_INTERRUPT    = 0b00000010;
        /// Whether to set the third bit in the status register. See
        /// [ControllerStatus::SYSTEM_FLAG].
        const SET_SYSTEM_FLAG           = 0b00000100;
        /// Whether to disable the keyboard interface by driving the clock line low.
        const DISABLE_KEYBOARD          = 0b00010000;
        /// Whether to disable the mouse interface by driving the clock line low.
        const DISABLE_MOUSE             = 0b00100000;
        /// Whether to enable translation of keyboard scancodes to set 1.
        const ENABLE_TRANSLATE          = 0b01000000;
    }
}

bitflags! {
    pub struct ControllerInput: u8 {
        /// Keyboard input data line.
        const KEYBOARD_DATA        = 0b00000001;
        /// Mouse input data line.
        const MOUSE_DATA           = 0b00000010;
        /// Whether an extra 256 KB of system board RAM is enabled.
        const ENABLE_EXTRA_RAM     = 0b00010000;
        /// Manufacturing jumper setting for keyboard testing.
        const NO_MANUFACTURING_JUMPER = 0b00100000;
        /// Keyboard display type bit.
        const MONOCHROME_DISPLAY   = 0b01000000;
        /// Whether keyboard functionality is enabled.
        const KEYBOARD_ENABLED     = 0b10000000;
    }
}

bitflags! {
    pub struct ControllerOutput: u8 {
        /// Whether to reset the CPU.
        const SYSTEM_RESET       = 0b00000001;
        /// Whether the 20th address line is enabled.
        const A20_GATE           = 0b00000010;
        /// Whether mouse data line is pulled low.
        const MOUSE_DATA         = 0b00000100;
        /// Whether mouse clock line is pulled low.
        const MOUSE_CLOCK        = 0b00001000;
        /// Whether keyboard triggers IRQ1 when input buffer is full.
        const KEYBOARD_INTERRUPT = 0b00010000;
        /// Whether mouse triggers IRQ12 when input buffer is full.
        const MOUSE_INTERRUPT    = 0b00100000;
        /// Whether keyboard clock line is pulled low.
        const KEYBOARD_CLOCK     = 0b01000000;
        /// Whether keyboard data line is pulled low.
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

bitflags! {
    pub struct MouseStatus: u8 {
        const RIGHT_BUTTON_PRESSED   = 0b00000001;
        const MIDDLE_BUTTON_PRESSED  = 0b00000010;
        const LEFT_BUTTON_PRESSED    = 0b00000100;
        const SCALING_2_TO_1         = 0b00010000;
        const DATA_REPORTING_ENABLED = 0b00100000;
        const REMOTE_MODE_ENABLED    = 0b01000000;
    }
}

bitflags! {
    pub struct MouseMovement: u8 {
        const LEFT_BUTTON_PRESSED   = 0b00000001;
        const RIGHT_BUTTON_PRESSED  = 0b00000010;
        const MIDDLE_BUTTON_PRESSED = 0b00000100;
        const X_SIGN_BIT            = 0b00010000;
        const Y_SIGN_BIT            = 0b00100000;
        const X_OVERFLOW            = 0b01000000;
        const Y_OVERFLOW            = 0b10000000;
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
