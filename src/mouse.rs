use crate::{
    controller::Controller, error::MouseError, COMMAND_ACKNOWLEDGED, RESEND, SELF_TEST_FAILED,
    SELF_TEST_PASSED,
};

type Result<T> = core::result::Result<T, MouseError>;

#[repr(u8)]
enum Command {
    Reset = 0xff,
    ResendLastByte = 0xfe,
    SetDefaults = 0xf6,
    DisableDataReporting = 0xf5,
    EnableDataReporting = 0xf4,
    SetSampleRate = 0xf3,
    GetDeviceID = 0xf2,
    SetRemoteMode = 0xf0,
    SetWrapMode = 0xee,
    ResetWrapMode = 0xec,
    ReadData = 0xeb,
    SetStreamMode = 0xea,
    StatusRequest = 0xe9,
    SetResolution = 0xe8,
    SetScaling2To1 = 0xe7,
    SetScaling1To1 = 0xe6,
}

#[derive(Debug)]
pub struct Mouse {
    controller: Controller,
}

impl Mouse {
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
            COMMAND_ACKNOWLEDGED => Ok(()),
            RESEND => Err(MouseError::Resend),
            other => Err(MouseError::InvalidResponse(other)),
        }
    }

    fn write_command(&mut self, command: Command, data: Option<u8>) -> Result<()> {
        self.controller.write_mouse(command as u8);
        self.check_response()?;
        if let Some(data) = data {
            self.controller.write_data(data as u8)?;
            self.check_response()?;
        }
        Ok(())
    }

    pub fn resend_last_byte(&mut self) -> Result<u8> {
        self.controller.write_mouse(Command::ResendLastByte as u8)?;
        // TODO: 0xfe won't ever be sent in response. Check if this is true for keyboard too
        Ok(self.controller.read_data()?)
    }

    pub fn reset_and_self_test(&mut self) -> Result<()> {
        self.controller.write_mouse(Command::Reset as u8)?;
        match self.controller.read_data()? {
            SELF_TEST_PASSED => Ok(()),
            SELF_TEST_FAILED => Err(MouseError::SelfTestFailed),
            RESEND => Err(MouseError::Resend),
            other => Err(MouseError::InvalidResponse(other)),
        }
    }
}

impl Default for Mouse {
    fn default() -> Self {
        Self::new()
    }
}
