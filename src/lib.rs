// References:
//   https://www.avrfreaks.net/sites/default/files/PS2%20Keyboard.pdf
//   http://www.s100computers.com/My%20System%20Pages/MSDOS%20Board/PC%20Keyboard.pdf

use x86_64::instructions::port::Port;

use self::flags::ControllerStatus;

mod flags;

const DATA_PORT: u16 = 0x60;
const COMMAND_REGISTER: u16 = 0x64;

// TODO: Timeout for reads/writes

pub struct PS2Controller {
    command_register: Port<u8>,
    data_port: Port<u8>,
}

impl PS2Controller {
    pub const fn new() -> Self {
        Self {
            command_register: Port::new(COMMAND_REGISTER),
            data_port: Port::new(DATA_PORT),
        }
    }

    pub fn get_status(&mut self) -> ControllerStatus {
        ControllerStatus::from_bits_truncate(unsafe { self.command_register.read() })
    }

    fn write_command(&mut self, command: u8) {
        while self.get_status().contains(ControllerStatus::INPUT_FULL) {}
        unsafe { self.command_register.write(command) }
    }

    pub fn read_data(&mut self) -> u8 {
        while !self.get_status().contains(ControllerStatus::OUTPUT_FULL) {}
        unsafe { self.data_port.read() }
    }

    pub fn write_data(&mut self, data: u8) {
        while self.get_status().contains(ControllerStatus::INPUT_FULL) {}
        unsafe { self.data_port.write(data) }
    }

    pub fn controller_command(&mut self, command: u8, data: Option<u8>) {
        self.write_command(command);
        data.map(|data| self.write_data(data));
    }

    pub fn device_command(&mut self, command: u8, data: Option<u8>) {
        self.write_data(command);
        data.map(|data| self.write_data(data));
    }
}
