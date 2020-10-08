use crate::{controller::Controller, error::KeyboardError};

const BUFFER_OVERRUN: u8 = 0x00;
const SELF_TEST_PASSED: u8 = 0xaa;
const ECHO: u8 = 0xee;
const COMMAND_ACKNOWLEDGED: u8 = 0xfa;
const SELF_TEST_FAILED: u8 = 0xfc;
const RESEND: u8 = 0xfe;
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

pub struct Keyboard {
    controller: Controller,
}

impl Keyboard {
    pub const fn new() -> Self {
        Self {
            controller: Controller::new(),
        }
    }

    pub fn enable_interrupts(&mut self) {
        self.controller.enable_interrupts();
    }

    pub fn disable_interrupts(&mut self) {
        self.controller.disable_interrupts();
    }

    fn check_response(&mut self) -> Result<()> {
        match self.controller.read_data()? {
            BUFFER_OVERRUN => Err(KeyboardError::KeyDetectionError),
            SELF_TEST_PASSED => Ok(()),
            ECHO => Ok(()),
            COMMAND_ACKNOWLEDGED => Ok(()),
            SELF_TEST_FAILED => Err(KeyboardError::SelfTestFailed),
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
}
