use bitflags::bitflags;
use x86_64::instructions::port::Port;

const DATA_PORT: u16 = 0x60;
const COMMAND_REGISTER: u16 = 0x64;

bitflags! {
    struct ControllerStatus: u8 {
        const OUTPUT_FULL        = 0b00000001;
        const INPUT_FULL         = 0b00000010;
        const SYSTEM_FLAG        = 0b00000100;
        const INPUT_IS_COMMAND   = 0b00001000;
        const KEYBOARD_LOCK      = 0b00010000;
        const MOUSE_OUTPUT_FULL  = 0b00100000;
        const TIMEOUT_ERR        = 0b01000000;
        const PARITY_ERR         = 0b10000000;
    }
}

bitflags! {
    struct ControllerConfig: u8 {
        const KEYBOARD_INTERRUPT = 0b00000001;
        const MOUSE_INTERRUPT    = 0b00000010;
        const SYSTEM_FLAG        = 0b00000100;
        const DISABLE_KEYBOARD   = 0b00010000;
        const DISABLE_MOUSE      = 0b00100000;
        const TRANSLATE          = 0b01000000;
    }
}

bitflags! {
    struct ControllerOutput: u8 {
        const SYSTEM_RESET         = 0b00000001;
        const A20_GATE             = 0b00000010;
        const MOUSE_CLOCK          = 0b00000100;
        const MOUSE_DATA           = 0b00001000;
        const KEYBOARD_OUTPUT_FULL = 0b00010000;
        const MOUSE_OUTPUT_FULL    = 0b00100000;
        const KEYBOARD_CLOCK       = 0b01000000;
        const KEYBOARD_DATA        = 0b10000000;
    }
}

// TODO: Timeout for reads/writes

pub struct PS2Controller {
    command_register: Port<u8>,
    data_port: Port<u8>,
}

impl PS2Controller {
    pub fn new() -> Self {
        Self {
            command_register: Port::new(COMMAND_REGISTER),
            data_port: Port::new(DATA_PORT),
        }
    }

    pub fn status(&mut self) -> u8 {
        unsafe { self.command_register.read() }
    }

    fn is_output_buffer_full(&mut self) -> bool {
        self.status() & 0b00000001 != 0
    }

    fn is_input_buffer_full(&mut self) -> bool {
        self.status() & 0b00000010 != 0
    }

    fn is_command_pending(&mut self) -> bool {
        self.is_input_buffer_full() && self.status() & 0b00001000 != 0
    }

    fn write_command(&mut self, command: u8) {
        while self.is_command_pending() {}
        unsafe { self.command_register.write(command) }
    }

    pub fn read_data(&mut self) -> u8 {
        while !self.is_output_buffer_full() {}
        unsafe { self.data_port.read() }
    }

    pub fn write_data(&mut self, data: u8) {
        while self.is_input_buffer_full() {}
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
