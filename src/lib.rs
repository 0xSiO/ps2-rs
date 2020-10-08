#![no_std]

// References:
//   https://wiki.osdev.org/%228042%22_PS/2_Controller
//   https://wiki.osdev.org/PS2_Keyboard
//   https://wiki.osdev.org/PS/2_Mouse
//   https://wiki.osdev.org/Mouse_Input
//   https://www.avrfreaks.net/sites/default/files/PS2%20Keyboard.pdf
//   http://www.s100computers.com/My%20System%20Pages/MSDOS%20Board/PC%20Keyboard.pdf

pub use self::controller::Controller;

mod controller;
mod response;

pub mod error;
pub mod flags;

const DATA_PORT: u16 = 0x60;
const COMMAND_REGISTER: u16 = 0x64;
const TIMEOUT: u16 = 10_000;
