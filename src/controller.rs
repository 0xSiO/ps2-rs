use x86_64::instructions::port::Port;

use crate::{
    error::ControllerError,
    flags::{ControllerConfig, ControllerInput, ControllerOutput, ControllerStatus},
};

const DATA_PORT: u16 = 0x60;
const COMMAND_REGISTER: u16 = 0x64;
const TIMEOUT: u16 = 10_000;

type Result<T> = core::result::Result<T, ControllerError>;

#[repr(u8)]
pub(crate) enum Command {
    ReadInternalRam = 0x20,
    WriteInternalRam = 0x60,
    DisableMouse = 0xa7,
    EnableMouse = 0xa8,
    TestMouse = 0xa9,
    TestController = 0xaa,
    TestKeyboard = 0xab,
    DiagnosticDump = 0xac,
    DisableKeyboard = 0xad,
    EnableKeyboard = 0xae,
    ReadControllerInput = 0xc0,
    WriteLowInputNibbleToStatus = 0xc1,
    WriteHighInputNibbleToStatus = 0xc2,
    ReadControllerOutput = 0xd0,
    WriteControllerOutput = 0xd1,
    WriteKeyboardBuffer = 0xd2,
    WriteMouseBuffer = 0xd3,
    WriteMouse = 0xd4,
    PulseOutput = 0xf0,
}

#[derive(Debug)]
pub struct Controller {
    non_blocking_read: bool,
    command_register: Port<u8>,
    data_port: Port<u8>,
}

impl Controller {
    pub const fn new() -> Self {
        Self {
            non_blocking_read: false,
            command_register: Port::new(COMMAND_REGISTER),
            data_port: Port::new(DATA_PORT),
        }
    }

    pub fn disable_blocking_read(&mut self) {
        self.non_blocking_read = true;
    }

    pub fn enable_blocking_read(&mut self) {
        self.non_blocking_read = false;
    }

    pub fn read_status(&mut self) -> ControllerStatus {
        ControllerStatus::from_bits_truncate(unsafe { self.command_register.read() })
    }

    fn wait_for_read(&mut self) -> Result<()> {
        if self.non_blocking_read {
            return Err(ControllerError::WouldBlock);
        }

        let mut timeout = TIMEOUT;
        while !self.read_status().contains(ControllerStatus::OUTPUT_FULL) && timeout > 0 {
            timeout -= 1;
        }

        if timeout == 0 {
            Err(ControllerError::Timeout)
        } else {
            Ok(())
        }
    }

    fn wait_for_write(&mut self) -> Result<()> {
        let mut timeout = TIMEOUT;
        while self.read_status().contains(ControllerStatus::INPUT_FULL) && timeout > 0 {
            timeout -= 1;
        }

        if timeout == 0 {
            Err(ControllerError::Timeout)
        } else {
            Ok(())
        }
    }

    pub(crate) fn write_command(&mut self, command: Command) -> Result<()> {
        self.wait_for_write()?;
        Ok(unsafe { self.command_register.write(command as u8) })
    }

    pub(crate) fn read_data(&mut self) -> Result<u8> {
        self.wait_for_read()?;
        Ok(unsafe { self.data_port.read() })
    }

    pub(crate) fn write_data(&mut self, data: u8) -> Result<()> {
        self.wait_for_write()?;
        Ok(unsafe { self.data_port.write(data) })
    }

    pub fn read_internal_ram(&mut self, byte_number: u8) -> Result<u8> {
        // Limit from 0 - 31, start command byte at 0x20
        let command = Command::ReadInternalRam as u8 | byte_number & 0x1f;
        // Since we did some bit fiddling, we can't use write_command
        self.wait_for_write()?;
        unsafe {
            self.command_register.write(command as u8);
        }
        self.read_data()
    }

    pub fn write_internal_ram(&mut self, byte_number: u8, data: u8) -> Result<()> {
        // Limit from 0 - 31, start command byte at 0x60
        let command = Command::WriteInternalRam as u8 | byte_number & 0x1f;
        // Since we did some bit fiddling, we can't use write_command
        self.wait_for_write()?;
        unsafe {
            self.command_register.write(command as u8);
        }
        self.write_data(data)
    }

    pub fn read_config(&mut self) -> Result<ControllerConfig> {
        Ok(ControllerConfig::from_bits_truncate(
            self.read_internal_ram(0)?,
        ))
    }

    pub fn disable_mouse(&mut self) -> Result<()> {
        self.write_command(Command::DisableMouse)
    }

    pub fn enable_mouse(&mut self) -> Result<()> {
        self.write_command(Command::EnableMouse)
    }

    pub fn test_mouse(&mut self) -> Result<()> {
        self.write_command(Command::TestMouse)?;
        match self.read_data()? {
            0x00 => Ok(()),
            err => Err(ControllerError::TestFailed { response: err }),
        }
    }

    pub fn test_controller(&mut self) -> Result<()> {
        self.write_command(Command::TestController)?;
        match self.read_data()? {
            0x55 => Ok(()),
            err => Err(ControllerError::TestFailed { response: err }),
        }
    }

    pub fn test_keyboard(&mut self) -> Result<()> {
        self.write_command(Command::TestKeyboard)?;
        match self.read_data()? {
            0x00 => Ok(()),
            err => Err(ControllerError::TestFailed { response: err }),
        }
    }

    // TODO: Test this, eventually. I wasn't able to get it working with any of my devices
    pub fn diagnostic_dump(&mut self) -> Result<[u8; 32]> {
        self.write_command(Command::DiagnosticDump)?;
        let mut result = [0; 32];
        for byte in result.iter_mut() {
            *byte = self.read_data()?;
        }
        Ok(result)
    }

    pub fn disable_keyboard(&mut self) -> Result<()> {
        self.write_command(Command::DisableKeyboard)
    }

    pub fn enable_keyboard(&mut self) -> Result<()> {
        self.write_command(Command::EnableKeyboard)
    }

    pub fn read_controller_input(&mut self) -> Result<ControllerInput> {
        self.write_command(Command::ReadControllerInput)?;
        Ok(ControllerInput::from_bits_truncate(self.read_data()?))
    }

    pub fn write_input_low_nibble_to_status(&mut self) -> Result<()> {
        self.write_command(Command::WriteLowInputNibbleToStatus)
    }

    pub fn write_input_high_nibble_to_status(&mut self) -> Result<()> {
        self.write_command(Command::WriteHighInputNibbleToStatus)
    }

    pub fn read_controller_output(&mut self) -> Result<ControllerOutput> {
        self.write_command(Command::ReadControllerOutput)?;
        Ok(ControllerOutput::from_bits_truncate(self.read_data()?))
    }

    pub fn write_controller_output(&mut self, output: ControllerOutput) -> Result<()> {
        self.write_command(Command::WriteControllerOutput)?;
        self.write_data(output.bits())
    }

    pub fn write_keyboard_buffer(&mut self, data: u8) -> Result<()> {
        self.write_command(Command::WriteKeyboardBuffer)?;
        self.write_data(data)
    }

    pub fn write_mouse_buffer(&mut self, data: u8) -> Result<()> {
        self.write_command(Command::WriteMouseBuffer)?;
        self.write_data(data)
    }

    pub fn write_mouse(&mut self, data: u8) -> Result<()> {
        self.write_command(Command::WriteMouse)?;
        self.write_data(data)
    }

    pub fn pulse_output_low_nibble(&mut self, data: u8) -> Result<()> {
        // Make the high nibble all 1's
        let command = Command::PulseOutput as u8 | data;
        // Since we did some bit fiddling, we can't use write_command
        self.wait_for_write()?;
        unsafe {
            self.command_register.write(command as u8);
        }
        Ok(())
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}
