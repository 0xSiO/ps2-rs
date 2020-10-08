#![no_std]

// References:
//   https://web.archive.org/web/20091124055529/http://www.computer-engineering.org/index.php?title=Main_Page
//   https://wiki.osdev.org/%228042%22_PS/2_Controller
//   https://wiki.osdev.org/PS2_Keyboard
//   https://wiki.osdev.org/PS/2_Mouse
//   https://wiki.osdev.org/Mouse_Input
//   http://www.s100computers.com/My%20System%20Pages/MSDOS%20Board/PC%20Keyboard.pdf

pub use self::{controller::Controller, keyboard::Keyboard};

mod controller;
mod keyboard;

pub mod error;
pub mod flags;

const DATA_PORT: u16 = 0x60;
const COMMAND_REGISTER: u16 = 0x64;
const TIMEOUT: u16 = 10_000;
