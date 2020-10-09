use crate::{
    controller::Controller,
    error::{ControllerError, KeyboardError},
    flags::KeyboardLeds,
};

pub use self::keyboard_type::KeyboardType;

mod keyboard_type;

const BUFFER_OVERRUN: u8 = 0x00;
const SELF_TEST_PASSED: u8 = 0xaa;
const ECHO: u8 = 0xee;
const COMMAND_ACKNOWLEDGED: u8 = 0xfa;
const SELF_TEST_FAILED: u8 = 0xfc;
const RESEND: u8 = 0xfe;
const KEY_DETECTION_ERROR: u8 = 0xff;

// Scale is 0x00 (30 Hz) to 0x1f (2 Hz). I'm assuming it's linear, but I could be wrong.
// For desired frequency f and scale factor s, the approximate byte needed is (30 - f) / s
const TYPEMATIC_RATE_SCALE_FACTOR: f64 = 28.0 / 31.0;
// Valid typematic delay values in milliseconds
const VALID_DELAYS: [u16; 4] = [250, 500, 750, 1000];

type Result<T> = core::result::Result<T, KeyboardError>;

#[repr(u8)]
enum Command {
    SetLeds = 0xed,
    Echo = 0xee,
    GetOrSetScancode = 0xf0,
    IdentifyKeyboard = 0xf2,
    SetTypematicRateAndDelay = 0xf3,
    EnableScanning = 0xf4,
    DisableScanning = 0xf5,
    SetDefaults = 0xf6,
    SetAllKeysTypematic = 0xf7,
    SetAllKeysMakeBreak = 0xf8,
    SetAllKeysMakeOnly = 0xf9,
    SetAllKeysTypematicAndMakeBreak = 0xfa,
    SetKeyTypematic = 0xfb,
    SetKeyMakeBreak = 0xfc,
    SetKeyMakeOnly = 0xfd,
    ResendLastByte = 0xfe,
    ResetAndSelfTest = 0xff,
}

#[derive(Debug)]
pub struct Keyboard {
    controller: Controller,
}

impl Keyboard {
    pub const fn new() -> Self {
        Self {
            controller: Controller::new(),
        }
    }

    pub fn disable_blocking_read(&mut self) {
        self.controller.disable_blocking_read();
    }

    pub fn enable_blocking_read(&mut self) {
        self.controller.enable_blocking_read();
    }

    fn check_response(&mut self) -> Result<()> {
        match self.controller.read_data()? {
            BUFFER_OVERRUN => Err(KeyboardError::KeyDetectionError),
            COMMAND_ACKNOWLEDGED => Ok(()),
            RESEND => Err(KeyboardError::Resend),
            KEY_DETECTION_ERROR => Err(KeyboardError::KeyDetectionError),
            other => Err(KeyboardError::InvalidResponse(other)),
        }
    }

    fn write_command(&mut self, command: Command, data: Option<u8>) -> Result<()> {
        self.controller.write_data(command as u8)?;
        self.check_response()?;
        if let Some(data) = data {
            self.controller.write_data(data as u8)?;
            self.check_response()?;
        }
        Ok(())
    }

    pub fn set_leds(&mut self, leds: KeyboardLeds) -> Result<()> {
        self.write_command(Command::SetLeds, Some(leds.bits()))
    }

    pub fn echo(&mut self) -> Result<()> {
        self.controller.write_data(Command::Echo as u8)?;
        match self.controller.read_data()? {
            ECHO => Ok(()),
            RESEND => Err(KeyboardError::Resend),
            other => Err(KeyboardError::InvalidResponse(other)),
        }
    }

    pub fn get_scancode_set(&mut self) -> Result<u8> {
        self.write_command(Command::GetOrSetScancode, Some(0))?;
        Ok(self.controller.read_data()?)
    }

    pub fn set_scancode_set(&mut self, scancode_set: u8) -> Result<()> {
        self.write_command(Command::GetOrSetScancode, Some(scancode_set))?;
        Ok(())
    }

    pub fn identify_keyboard(&mut self) -> Result<KeyboardType> {
        // First check to see if the command was acknowledged
        match self.write_command(Command::IdentifyKeyboard, None) {
            Ok(()) => {}
            // XT keyboards don't acknowledge this command
            Err(KeyboardError::Resend) => return Ok(KeyboardType::XT),
            Err(other) => return Err(other.into()),
        }

        // Now check for a second byte - AT keyboards won't give one
        match self.controller.read_data() {
            Ok(first_byte) => {
                let second_byte = self.controller.read_data()?;
                Ok(KeyboardType::from((first_byte, second_byte)))
            }
            Err(ControllerError::Timeout) => Ok(KeyboardType::ATWithTranslation),
            Err(other) => return Err(other.into()),
        }
    }

    pub fn set_typematic_rate_and_delay(&mut self, repeat_rate: f64, delay: u16) -> Result<()> {
        if repeat_rate < 2.0 || repeat_rate > 30.0 {
            return Err(KeyboardError::InvalidTypematicFrequency(repeat_rate));
        }
        if !VALID_DELAYS.contains(&delay) {
            return Err(KeyboardError::InvalidTypematicDelay(delay));
        }

        let scaled_rate: f64 = (30.0 - repeat_rate) / TYPEMATIC_RATE_SCALE_FACTOR;
        let encoded_rate = 0b00011111 & (libm::round(scaled_rate) as u8);

        // Ok to unwrap since we already checked for existence in VALID_DELAYS. Also safe
        // to cast to u8 since VALID_DELAYS has only 4 elements
        let encoded_delay = VALID_DELAYS.iter().position(|&n| n == delay).unwrap() as u8;

        self.write_command(
            Command::SetTypematicRateAndDelay,
            Some(encoded_delay << 5 | encoded_rate),
        )
    }

    pub fn enable_scanning(&mut self) -> Result<()> {
        self.write_command(Command::EnableScanning, None)
    }

    pub fn disable_scanning(&mut self) -> Result<()> {
        self.write_command(Command::DisableScanning, None)
    }

    pub fn set_defaults(&mut self) -> Result<()> {
        self.write_command(Command::SetDefaults, None)
    }

    pub fn set_all_keys_typematic(&mut self) -> Result<()> {
        self.write_command(Command::SetAllKeysTypematic, None)
    }

    pub fn set_all_keys_make_break(&mut self) -> Result<()> {
        self.write_command(Command::SetAllKeysMakeBreak, None)
    }

    pub fn set_all_keys_make_only(&mut self) -> Result<()> {
        self.write_command(Command::SetAllKeysMakeOnly, None)
    }

    pub fn set_all_keys_typematic_make_break(&mut self) -> Result<()> {
        self.write_command(Command::SetAllKeysTypematicAndMakeBreak, None)
    }

    pub fn set_key_typematic(&mut self, scancode: u8) -> Result<()> {
        self.write_command(Command::SetKeyTypematic, Some(scancode))
    }

    pub fn set_key_make_break(&mut self, scancode: u8) -> Result<()> {
        self.write_command(Command::SetKeyMakeBreak, Some(scancode))
    }

    pub fn set_key_make_only(&mut self, scancode: u8) -> Result<()> {
        self.write_command(Command::SetKeyMakeOnly, Some(scancode))
    }

    pub fn resend_last_byte(&mut self) -> Result<u8> {
        self.controller.write_data(Command::ResendLastByte as u8)?;
        match self.controller.read_data()? {
            RESEND => Err(KeyboardError::Resend),
            byte => Ok(byte),
        }
    }

    pub fn reset_and_self_test(&mut self) -> Result<()> {
        self.controller
            .write_data(Command::ResetAndSelfTest as u8)?;
        match self.controller.read_data()? {
            SELF_TEST_PASSED => Ok(()),
            SELF_TEST_FAILED => Err(KeyboardError::SelfTestFailed),
            RESEND => Err(KeyboardError::Resend),
            other => Err(KeyboardError::InvalidResponse(other)),
        }
    }
}
