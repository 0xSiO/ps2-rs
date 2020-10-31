use crate::{
    controller::Controller, error::MouseError, flags::MouseStatus, COMMAND_ACKNOWLEDGED, RESEND,
    SELF_TEST_FAILED, SELF_TEST_PASSED,
};

// Valid resolution values in counts per mm
const VALID_RESOLUTIONS: [u8; 4] = [1, 2, 4, 8];

type Result<T> = core::result::Result<T, MouseError>;

#[repr(u8)]
enum Command {
    SetScaling1To1 = 0xe6,
    SetScaling2To1 = 0xe7,
    SetResolution = 0xe8,
    StatusRequest = 0xe9,
    SetStreamMode = 0xea,
    ReadData = 0xeb,
    ResetWrapMode = 0xec,
    SetWrapMode = 0xee,
    SetRemoteMode = 0xf0,
    GetDeviceID = 0xf2,
    SetSampleRate = 0xf3,
    EnableDataReporting = 0xf4,
    DisableDataReporting = 0xf5,
    SetDefaults = 0xf6,
    ResendLastByte = 0xfe,
    ResetAndSelfTest = 0xff,
}

#[repr(u8)]
pub enum Resolution {
    OneCountPerMM = 0x00,
    TwoCountPerMM = 0x01,
    FourCountPerMM = 0x02,
    EightCountPerMM = 0x03,
}

#[derive(Debug)]
pub struct Mouse {
    controller: Controller,
}

// TODO: Support Intellimouse extensions
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

    unsafe fn check_response(&mut self) -> Result<()> {
        match self.controller.read_data()? {
            COMMAND_ACKNOWLEDGED => Ok(()),
            RESEND => Err(MouseError::Resend),
            other => Err(MouseError::InvalidResponse(other)),
        }
    }

    unsafe fn write_command(&mut self, command: Command, data: Option<u8>) -> Result<()> {
        self.controller.write_mouse(command as u8)?;
        self.check_response()?;
        if let Some(data) = data {
            self.controller.write_data(data as u8)?;
            self.check_response()?;
        }
        Ok(())
    }

    pub unsafe fn set_scaling_one_to_one(&mut self) -> Result<()> {
        self.write_command(Command::SetScaling1To1, None)
    }

    pub unsafe fn set_scaling_two_to_one(&mut self) -> Result<()> {
        self.write_command(Command::SetScaling2To1, None)
    }

    pub unsafe fn set_resolution(&mut self, resolution: Resolution) -> Result<()> {
        self.write_command(Command::SetResolution, Some(resolution as u8))
    }

    pub unsafe fn request_status(&mut self) -> Result<(MouseStatus, u8, u8)> {
        self.write_command(Command::StatusRequest, None)?;
        let status = MouseStatus::from_bits_truncate(self.controller.read_data()?);
        // TODO: Parse this into an enum variant
        let resolution = self.controller.read_data()?;
        let sample_rate = self.controller.read_data()?;
        Ok((status, resolution, sample_rate))
    }

    pub unsafe fn resend_last_byte(&mut self) -> Result<u8> {
        self.controller.write_mouse(Command::ResendLastByte as u8)?;
        // TODO: 0xfe won't ever be sent in response. Check if this is true for keyboard too
        Ok(self.controller.read_data()?)
    }

    pub unsafe fn reset_and_self_test(&mut self) -> Result<()> {
        self.write_command(Command::ResetAndSelfTest, None)?;
        let result = match self.controller.read_data()? {
            SELF_TEST_PASSED => Ok(()),
            SELF_TEST_FAILED => Err(MouseError::SelfTestFailed),
            RESEND => Err(MouseError::Resend),
            other => Err(MouseError::InvalidResponse(other)),
        };
        let _device_id = self.controller.read_data()?;
        result
    }
}

impl Default for Mouse {
    fn default() -> Self {
        Self::new()
    }
}
