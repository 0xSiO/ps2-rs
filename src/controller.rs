use x86_64::instructions::port::Port;

use crate::{
    flags::{Config, Input, Output, Status},
    response::Response,
};

const DATA_PORT: u16 = 0x60;
const COMMAND_REGISTER: u16 = 0x64;

// TODO: Timeout for reads/writes

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

    fn write_command(&mut self, command: u8) {
        while self.read_status().contains(Status::INPUT_FULL) {}
        unsafe { self.command_register.write(command) }
    }

    fn read_data(&mut self) -> u8 {
        if self.use_interrupts {
            return 0;
        }
        while !self.read_status().contains(Status::OUTPUT_FULL) {}
        unsafe { self.data_port.read() }
    }

    fn write_data(&mut self, data: u8) {
        while self.read_status().contains(Status::INPUT_FULL) {}
        unsafe { self.data_port.write(data) }
    }

    pub fn read_internal_ram(&mut self, byte_number: u8) -> u8 {
        // Limit from 0 - 31, start command byte at 0x20
        let command = byte_number & 0x1f | 0x20;
        self.write_command(command);
        self.read_data()
    }

    pub fn write_internal_ram(&mut self, byte_number: u8, data: u8) {
        // Limit from 0 - 31, start command byte at 0x60
        let command = byte_number & 0x1f | 0x60;
        self.write_command(command);
        self.write_data(data);
    }

    pub fn read_config(&mut self) -> Config {
        Config::from_bits_truncate(self.read_internal_ram(0))
    }

    pub fn disable_mouse(&mut self) {
        self.write_command(0xa7);
    }

    pub fn enable_mouse(&mut self) {
        self.write_command(0xa8);
    }

    // TODO: Create test responses

    pub fn test_mouse(&mut self) -> u8 {
        self.write_command(0xa9);
        self.read_data()
    }

    pub fn test_controller(&mut self) -> u8 {
        self.write_command(0xaa);
        self.read_data()
    }

    pub fn test_keyboard(&mut self) -> u8 {
        self.write_command(0xab);
        self.read_data()
    }

    pub fn diagnostic_dump(&mut self) {
        self.write_command(0xac);
        // TODO: return array of all bytes
    }

    pub fn disable_keyboard(&mut self) {
        self.write_command(0xad);
    }

    pub fn enable_keyboard(&mut self) {
        self.write_command(0xae);
    }

    pub fn read_controller_input(&mut self) -> Input {
        self.write_command(0xc0);
        Input::from_bits_truncate(self.read_data())
    }

    pub fn read_status(&mut self) -> Status {
        Status::from_bits_truncate(unsafe { self.command_register.read() })
    }

    pub fn write_input_low_nibble_to_status(&mut self) {
        self.write_command(0xc1);
    }

    pub fn write_input_high_nibble_to_status(&mut self) {
        self.write_command(0xc2);
    }

    pub fn read_controller_output(&mut self) -> Output {
        self.write_command(0xd0);
        Output::from_bits_truncate(self.read_data())
    }

    pub fn write_controller_output(&mut self, output: Output) {
        self.write_command(0xd1);
        self.write_data(output.bits());
    }

    pub fn write_keyboard_buffer(&mut self, data: u8) {
        self.write_command(0xd2);
        self.write_data(data);
    }

    pub fn write_mouse_buffer(&mut self, data: u8) {
        self.write_command(0xd3);
        self.write_data(data);
    }

    pub fn write_mouse(&mut self, data: u8) {
        self.write_command(0xd4);
        self.write_data(data);
    }
}
