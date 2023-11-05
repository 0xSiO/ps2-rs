use crate::{
    controller::Controller,
    error::{ControllerError, KeyboardError},
    flags::KeyboardLedFlags,
    COMMAND_ACKNOWLEDGED, RESEND, SELF_TEST_FAILED, SELF_TEST_PASSED,
};

pub use self::keyboard_type::KeyboardType;

mod keyboard_type;

const BUFFER_OVERRUN: u8 = 0x00;
const ECHO: u8 = 0xee;
const KEY_DETECTION_ERROR: u8 = 0xff;

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

/// A PS/2 keyboard.
///
/// This provides the functionality of a typical PS/2 keyboard, as well as PS/2 devices
/// that act like keyboards, such as barcode scanners, card readers, fingerprint scanners,
/// etc.
///
/// # Examples
/// ```
/// use ps2::Controller;
///
/// let mut controller = unsafe { Controller::new() };
/// let mut keyboard = controller.keyboard();
/// ```
#[derive(Debug)]
pub struct Keyboard<'c> {
    controller: &'c mut Controller,
}

impl<'c> Keyboard<'c> {
    pub(crate) const fn new(controller: &'c mut Controller) -> Self {
        Self { controller }
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
            self.controller.write_data(data)?;
            self.check_response()?;
        }
        Ok(())
    }

    /// Set the state of the keyboard LEDs.
    pub fn set_leds(&mut self, leds: KeyboardLedFlags) -> Result<()> {
        self.write_command(Command::SetLeds, Some(leds.bits()))
    }

    /// Run a diagnostic echo command.
    pub fn echo(&mut self) -> Result<()> {
        self.controller.write_data(Command::Echo as u8)?;
        match self.controller.read_data()? {
            ECHO => Ok(()),
            RESEND => Err(KeyboardError::Resend),
            other => Err(KeyboardError::InvalidResponse(other)),
        }
    }

    /// Get the number corresponding to the current scancode set (1, 2, or 3).
    pub fn get_scancode_set(&mut self) -> Result<u8> {
        self.write_command(Command::GetOrSetScancode, Some(0))?;
        Ok(self.controller.read_data()?)
    }

    /// Set the scancode set number (1, 2, or 3).
    pub fn set_scancode_set(&mut self, scancode_set: u8) -> Result<()> {
        self.write_command(Command::GetOrSetScancode, Some(scancode_set))?;
        Ok(())
    }

    /// Attempt to obtain a device identifier for this keyboard.
    pub fn get_keyboard_type(&mut self) -> Result<KeyboardType> {
        // First check to see if the command was acknowledged
        match self.write_command(Command::IdentifyKeyboard, None) {
            Ok(()) => {}
            // XT keyboards don't acknowledge this command
            Err(KeyboardError::Resend) => return Ok(KeyboardType::XT),
            Err(other) => return Err(other),
        }

        // Now check for a second byte - AT keyboards won't give one
        match self.controller.read_data() {
            Ok(first_byte) => {
                let second_byte = self.controller.read_data()?;
                Ok(KeyboardType::from((first_byte, second_byte)))
            }
            Err(ControllerError::Timeout) => Ok(KeyboardType::ATWithTranslation),
            Err(other) => Err(other.into()),
        }
    }

    /// Set the typematic repeat rate and delay.
    ///
    /// Use the 'Repeat rate' and 'Delay' tables on
    /// [this](https://web.archive.org/web/20091128232820/http://www.computer-engineering.org/index.php?title=PS/2_Keyboard_Interface#Command_Set)
    /// page to create the configuration byte.
    pub fn set_typematic_rate_and_delay(&mut self, typematic_config: u8) -> Result<()> {
        self.write_command(
            Command::SetTypematicRateAndDelay,
            // Most significant bit is ignored
            Some(typematic_config & 0b01111111),
        )
    }

    /// Clear the data buffer and last typematic key, then enable scancodes.
    pub fn enable_scanning(&mut self) -> Result<()> {
        self.write_command(Command::EnableScanning, None)
    }

    /// Reset keyboard to power-on state and disable scancodes.
    pub fn disable_scanning(&mut self) -> Result<()> {
        self.write_command(Command::DisableScanning, None)
    }

    /// Reset keyboard to power-on state by clearing the data buffer and restoring all default key
    /// settings.
    pub fn set_defaults(&mut self) -> Result<()> {
        self.write_command(Command::SetDefaults, None)
    }

    /// Set all keys to typematic only. This only has an effect if scancode set 3 is in use.
    pub fn set_all_keys_typematic(&mut self) -> Result<()> {
        self.write_command(Command::SetAllKeysTypematic, None)
    }

    /// Set all keys to make/break only. This only has an effect if scancode set 3 is in use.
    pub fn set_all_keys_make_break(&mut self) -> Result<()> {
        self.write_command(Command::SetAllKeysMakeBreak, None)
    }

    /// Set all keys to make only. This only has an effect if scancode set 3 is in use.
    pub fn set_all_keys_make_only(&mut self) -> Result<()> {
        self.write_command(Command::SetAllKeysMakeOnly, None)
    }

    /// Set all keys to typematic and make/break. This only has an effect if scancode set 3 is in
    /// use.
    pub fn set_all_keys_typematic_make_break(&mut self) -> Result<()> {
        self.write_command(Command::SetAllKeysTypematicAndMakeBreak, None)
    }

    /// Set a specific key to typematic only. This only has an effect if scancode set 3 is in use.
    pub fn set_key_typematic(&mut self, scancode: u8) -> Result<()> {
        self.write_command(Command::SetKeyTypematic, Some(scancode))
    }

    /// Set a specific key to make/break only. This only has an effect if scancode set 3 is in use.
    pub fn set_key_make_break(&mut self, scancode: u8) -> Result<()> {
        self.write_command(Command::SetKeyMakeBreak, Some(scancode))
    }

    /// Set a specific key to make only. This only has an effect if scancode set 3 is in use.
    pub fn set_key_make_only(&mut self, scancode: u8) -> Result<()> {
        self.write_command(Command::SetKeyMakeOnly, Some(scancode))
    }

    /// Get the last byte sent by the keyboard.
    pub fn resend_last_byte(&mut self) -> Result<u8> {
        self.controller.write_data(Command::ResendLastByte as u8)?;
        match self.controller.read_data()? {
            RESEND => Err(KeyboardError::Resend),
            byte => Ok(byte),
        }
    }

    /// Reset the keyboard and perform a Basic Assurance Test.
    ///
    /// Returns [`KeyboardError::SelfTestFailed`] if the test fails.
    pub fn reset_and_self_test(&mut self) -> Result<()> {
        self.write_command(Command::ResetAndSelfTest, None)?;
        match self.controller.read_data()? {
            SELF_TEST_PASSED => Ok(()),
            SELF_TEST_FAILED => Err(KeyboardError::SelfTestFailed),
            RESEND => Err(KeyboardError::Resend),
            other => Err(KeyboardError::InvalidResponse(other)),
        }
    }
}
