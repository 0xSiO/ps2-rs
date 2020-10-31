use crate::{
    controller::Controller,
    error::MouseError,
    flags::{MouseMovement, MouseStatus},
    COMMAND_ACKNOWLEDGED, RESEND, SELF_TEST_FAILED, SELF_TEST_PASSED,
};

pub use self::mouse_type::MouseType;

mod mouse_type;

const VALID_RESOLUTIONS: [u8; 4] = [1, 2, 4, 8];
const VALID_SAMPLE_RATES: [u8; 7] = [10, 20, 40, 60, 80, 100, 200];

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

#[derive(Debug)]
pub struct Mouse<'c> {
    controller: &'c mut Controller,
}

// TODO: Support Intellimouse extensions
impl<'c> Mouse<'c> {
    // TODO: Read more about const_mut_refs feature if we want this function to be const
    pub(crate) fn new(controller: &'c mut Controller) -> Self {
        Self { controller }
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
        self.controller.write_mouse(command as u8)?;
        self.check_response()?;
        if let Some(data) = data {
            self.controller.write_data(data as u8)?;
            self.check_response()?;
        }
        Ok(())
    }

    pub fn set_scaling_one_to_one(&mut self) -> Result<()> {
        self.write_command(Command::SetScaling1To1, None)
    }

    pub fn set_scaling_two_to_one(&mut self) -> Result<()> {
        self.write_command(Command::SetScaling2To1, None)
    }

    pub fn set_resolution(&mut self, resolution: u8) -> Result<()> {
        if !VALID_RESOLUTIONS.contains(&resolution) {
            return Err(MouseError::InvalidResolution(resolution));
        }
        // Ok to unwrap since we already checked for existence in VALID_RESOLUTIONS.
        // Also safe to cast to u8 since VALID_RESOLUTIONS has only 4 elements
        let resolution_index = VALID_RESOLUTIONS
            .iter()
            .position(|&n| n == resolution)
            .unwrap() as u8;
        self.write_command(Command::SetResolution, Some(resolution_index))
    }

    pub fn request_status(&mut self) -> Result<(MouseStatus, u8, u8)> {
        self.write_command(Command::StatusRequest, None)?;
        let status = MouseStatus::from_bits_truncate(self.controller.read_data()?);
        let resolution = self.controller.read_data()?;
        let sample_rate = self.controller.read_data()?;
        if !VALID_RESOLUTIONS.contains(&resolution) {
            return Err(MouseError::InvalidResolution(resolution));
        }
        if !VALID_SAMPLE_RATES.contains(&sample_rate) {
            return Err(MouseError::InvalidSampleRate(sample_rate));
        }
        Ok((status, resolution, sample_rate))
    }

    pub fn set_stream_mode(&mut self) -> Result<()> {
        self.write_command(Command::SetStreamMode, None)
    }

    pub fn read_data(&mut self) -> Result<()> {
        self.write_command(Command::ReadData, None)?;
        // TODO: Process movement packet
        let _movement_flags = MouseMovement::from_bits_truncate(self.controller.read_data()?);
        let _x_movement = self.controller.read_data()?;
        let _y_movement = self.controller.read_data()?;
        Ok(())
    }

    pub fn reset_wrap_mode(&mut self) -> Result<()> {
        self.write_command(Command::ResetWrapMode, None)
    }

    pub fn set_wrap_mode(&mut self) -> Result<()> {
        self.write_command(Command::SetWrapMode, None)
    }

    pub fn set_remote_mode(&mut self) -> Result<()> {
        self.write_command(Command::SetRemoteMode, None)
    }

    pub fn get_device_id(&mut self) -> Result<MouseType> {
        self.write_command(Command::GetDeviceID, None)?;
        Ok(MouseType::from(self.controller.read_data()?))
    }

    pub fn set_sample_rate(&mut self, sample_rate: u8) -> Result<()> {
        if !VALID_SAMPLE_RATES.contains(&sample_rate) {
            return Err(MouseError::InvalidSampleRate(sample_rate));
        }
        self.write_command(Command::SetSampleRate, Some(sample_rate))
    }

    pub fn enable_data_reporting(&mut self) -> Result<()> {
        self.write_command(Command::EnableDataReporting, None)
    }

    pub fn disable_data_reporting(&mut self) -> Result<()> {
        self.write_command(Command::DisableDataReporting, None)
    }

    pub fn set_defaults(&mut self) -> Result<()> {
        self.write_command(Command::SetDefaults, None)
    }

    pub fn resend_last_byte(&mut self) -> Result<u8> {
        self.controller.write_mouse(Command::ResendLastByte as u8)?;
        // TODO: 0xfe won't ever be sent in response. Check if this is true for keyboard too
        Ok(self.controller.read_data()?)
    }

    pub fn reset_and_self_test(&mut self) -> Result<()> {
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
