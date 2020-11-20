use crate::{
    controller::Controller,
    error::MouseError,
    flags::{MouseMovementFlags, MouseStatusFlags},
    COMMAND_ACKNOWLEDGED, RESEND, SELF_TEST_FAILED, SELF_TEST_PASSED,
};

pub use self::mouse_type::MouseType;

mod mouse_type;

const VALID_RESOLUTIONS: [u8; 4] = [0, 1, 2, 3];
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
    ResendLastPacket = 0xfe,
    ResetAndSelfTest = 0xff,
}

/// A PS/2 mouse.
///
/// This provides the functionality of a typical PS/2 mouse, as well as PS/2 devices
/// that act like mice, such as touchpads or wireless mouse receivers.
///
/// # Examples
/// ```
/// use ps2::Controller;
///
/// let mut controller = unsafe { Controller::new() };
/// let mut mouse = controller.mouse();
/// ```
#[derive(Debug)]
pub struct Mouse<'c> {
    controller: &'c mut Controller,
}

// TODO: Support Intellimouse extensions
impl<'c> Mouse<'c> {
    pub(crate) const fn new(controller: &'c mut Controller) -> Self {
        Self { controller }
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

    /// Set the scaling of reported data to be 1:1.
    ///
    /// Read more about scaling
    /// [here](https://web.archive.org/web/20090325002201/http://www.computer-engineering.org/index.php?title=PS/2_Mouse_Interface#Inputs.2C_Resolution.2C_and_Scaling).
    pub fn set_scaling_one_to_one(&mut self) -> Result<()> {
        self.write_command(Command::SetScaling1To1, None)
    }

    /// Set the scaling of reported data to be 2:1.
    ///
    /// Read more about scaling
    /// [here](https://web.archive.org/web/20090325002201/http://www.computer-engineering.org/index.php?title=PS/2_Mouse_Interface#Inputs.2C_Resolution.2C_and_Scaling).
    pub fn set_scaling_two_to_one(&mut self) -> Result<()> {
        self.write_command(Command::SetScaling2To1, None)
    }

    /// Set mouse resolution.
    ///
    /// Valid values are `0` for 1 count/mm, `1` for 2 counts/mm, `2` for 4 counts/mm, or `3` for
    /// 8 counts/mm.
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

    /// Request a status packet from the mouse and reset the movement counters.
    ///
    /// The first byte returned is a bitfield, the second byte is the mouse resolution, and the
    /// third is the sample rate.
    pub fn get_status_packet(&mut self) -> Result<(MouseStatusFlags, u8, u8)> {
        self.write_command(Command::StatusRequest, None)?;
        let status = MouseStatusFlags::from_bits_truncate(self.controller.read_data()?);
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

    /// Reset mouse movement counters and enter streaming mode.
    ///
    /// Read more about modes
    /// [here](https://web.archive.org/web/20090325002201/http://www.computer-engineering.org/index.php?title=PS/2_Mouse_Interface#Modes_of_Operation).
    pub fn set_stream_mode(&mut self) -> Result<()> {
        self.write_command(Command::SetStreamMode, None)
    }

    /// Request a movement data packet from the mouse and reset the movement counters.
    ///
    /// The first byte returned is a bitfield, and the other two bytes are 9-bit two's complement
    /// integers for the horizontal and vertical movement offset relative to the position at which
    /// the last packet was sent.
    ///
    /// If you're writing an interrupt handler, see [`Mouse::read_data_packet`].
    pub fn request_data_packet(&mut self) -> Result<(MouseMovementFlags, i16, i16)> {
        self.write_command(Command::ReadData, None)?;
        Ok(self.read_data_packet()?)
    }

    /// Read an existing movement data packet directly from the data buffer.
    ///
    /// The first byte returned is a bitfield, and the other two bytes are 9-bit two's complement
    /// integers for the horizontal and vertical movement offset relative to the position at which
    /// the last packet was sent.
    ///
    /// This does **not** send any commands to the mouse. This is useful in interrupt handlers when
    /// we just want to read the data sent by the mouse.
    pub fn read_data_packet(&mut self) -> Result<(MouseMovementFlags, i16, i16)> {
        let movement_flags = MouseMovementFlags::from_bits_truncate(self.controller.read_data()?);
        let mut x_movement = self.controller.read_data()? as u16;
        let mut y_movement = self.controller.read_data()? as u16;

        if movement_flags.contains(MouseMovementFlags::X_SIGN_BIT) {
            x_movement |= 0xff00;
        }
        if movement_flags.contains(MouseMovementFlags::Y_SIGN_BIT) {
            y_movement |= 0xff00;
        }

        Ok((movement_flags, x_movement as i16, y_movement as i16))
    }

    /// Reset mouse movement counters and exit wrap mode, entering the mode the mouse was in
    /// previously.
    ///
    /// Read more about modes
    /// [here](https://web.archive.org/web/20090325002201/http://www.computer-engineering.org/index.php?title=PS/2_Mouse_Interface#Modes_of_Operation).
    pub fn reset_wrap_mode(&mut self) -> Result<()> {
        self.write_command(Command::ResetWrapMode, None)
    }

    /// Reset mouse movement counters and enter wrap mode.
    ///
    /// Read more about modes
    /// [here](https://web.archive.org/web/20090325002201/http://www.computer-engineering.org/index.php?title=PS/2_Mouse_Interface#Modes_of_Operation).
    pub fn set_wrap_mode(&mut self) -> Result<()> {
        self.write_command(Command::SetWrapMode, None)
    }

    /// Reset mouse movement counters and enter remote mode.
    ///
    /// Read more about modes
    /// [here](https://web.archive.org/web/20090325002201/http://www.computer-engineering.org/index.php?title=PS/2_Mouse_Interface#Modes_of_Operation).
    pub fn set_remote_mode(&mut self) -> Result<()> {
        self.write_command(Command::SetRemoteMode, None)
    }

    /// Attempt to obtain a device identifier for this mouse.
    pub fn get_mouse_type(&mut self) -> Result<MouseType> {
        self.write_command(Command::GetDeviceID, None)?;
        Ok(MouseType::from(self.controller.read_data()?))
    }

    /// Set the mouse sample rate and reset the movement counters.
    ///
    /// Valid rates are `10`, `20`, `40`, `60`, `80`, `100`, and `200`, in samples per second.
    pub fn set_sample_rate(&mut self, sample_rate: u8) -> Result<()> {
        if !VALID_SAMPLE_RATES.contains(&sample_rate) {
            return Err(MouseError::InvalidSampleRate(sample_rate));
        }
        self.write_command(Command::SetSampleRate, Some(sample_rate))
    }

    /// Enable data reporting and reset the movement counters.
    ///
    /// This only affects data reporting in stream mode.
    pub fn enable_data_reporting(&mut self) -> Result<()> {
        self.write_command(Command::EnableDataReporting, None)
    }

    /// Disable data reporting and reset the movement counters.
    ///
    /// This only affects data reporting in stream mode. Note that this only disables reporting,
    /// not sampling. Movement packets may still be read using [`Mouse::request_data_packet`].
    pub fn disable_data_reporting(&mut self) -> Result<()> {
        self.write_command(Command::DisableDataReporting, None)
    }

    /// Set defaults, clear movement counters, and enter stream mode.
    ///
    /// Default settings are as follows: sampling rate = 100 samples/second,
    /// resolution = 4 counts/mm, scaling = 1:1, data reporting disabled.
    pub fn set_defaults(&mut self) -> Result<()> {
        self.write_command(Command::SetDefaults, None)
    }

    /// Request that the mouse resend the last transmitted byte or packet.
    ///
    /// Currently, this does not return any data, since the resent data may be one or more bytes in
    /// length. It is the responsibility of the caller to consume these bytes using
    /// [`Controller::read_data`].
    pub fn resend_last_packet(&mut self) -> Result<()> {
        Ok(self
            .controller
            .write_mouse(Command::ResendLastPacket as u8)?)
    }

    /// Reset the mouse and perform a Basic Assurance Test.
    ///
    /// Returns [`MouseError::SelfTestFailed`] if the test fails.
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
