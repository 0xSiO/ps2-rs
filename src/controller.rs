use x86_64::instructions::port::Port;

use crate::{
    error::ControllerError,
    flags::{Config, Input, Output, Status},
};

const DATA_PORT: u16 = 0x60;
const COMMAND_REGISTER: u16 = 0x64;
const TIMEOUT: u16 = 10_000;

type Result<T> = core::result::Result<T, ControllerError>;

#[derive(Debug)]
#[repr(u8)]
enum Command {
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
    use_interrupts: bool,
    command_register: Port<u8>,
    data_port: Port<u8>,
}

impl Controller {
    pub const fn new(use_interrupts: bool) -> Self {
        Self {
            use_interrupts,
            command_register: Port::new(COMMAND_REGISTER),
            data_port: Port::new(DATA_PORT),
        }
    }

    pub fn read_status(&mut self) -> Status {
        Status::from_bits_truncate(unsafe { self.command_register.read() })
    }

    fn wait_for_read(&mut self) -> Result<()> {
        if self.use_interrupts {
            return Err(ControllerError::InterruptsEnabled);
        }

        let mut timeout = TIMEOUT;
        while !self.read_status().contains(Status::OUTPUT_FULL) && timeout > 0 {
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
        while self.read_status().contains(Status::INPUT_FULL) && timeout > 0 {
            timeout -= 1;
        }

        if timeout == 0 {
            Err(ControllerError::Timeout)
        } else {
            Ok(())
        }
    }

    fn write_command(&mut self, command: Command) -> Result<()> {
        self.wait_for_write()?;
        Ok(unsafe { self.command_register.write(command as u8) })
    }

    fn read_data(&mut self) -> Result<u8> {
        self.wait_for_read()?;
        Ok(unsafe { self.data_port.read() })
    }

    fn write_data(&mut self, data: u8) -> Result<()> {
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

    pub fn read_config(&mut self) -> Result<Config> {
        Ok(Config::from_bits_truncate(self.read_internal_ram(0)?))
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

    pub fn diagnostic_dump(&mut self) -> Result<()> {
        self.write_command(Command::DiagnosticDump)
        // TODO: return array of all bytes
    }

    pub fn disable_keyboard(&mut self) -> Result<()> {
        self.write_command(Command::DisableKeyboard)
    }

    pub fn enable_keyboard(&mut self) -> Result<()> {
        self.write_command(Command::EnableKeyboard)
    }

    pub fn read_controller_input(&mut self) -> Result<Input> {
        self.write_command(Command::ReadControllerInput)?;
        Ok(Input::from_bits_truncate(self.read_data()?))
    }

    pub fn write_input_low_nibble_to_status(&mut self) -> Result<()> {
        self.write_command(Command::WriteLowInputNibbleToStatus)
    }

    pub fn write_input_high_nibble_to_status(&mut self) -> Result<()> {
        self.write_command(Command::WriteHighInputNibbleToStatus)
    }

    pub fn read_controller_output(&mut self) -> Result<Output> {
        self.write_command(Command::ReadControllerOutput)?;
        Ok(Output::from_bits_truncate(self.read_data()?))
    }

    pub fn write_controller_output(&mut self, output: Output) -> Result<()> {
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
