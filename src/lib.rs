#![no_std]

// References:
//   https://web.archive.org/web/20091124055529/http://www.computer-engineering.org/index.php?title=Main_Page
//   https://web.archive.org/web/20200616182211/https://www.win.tue.nl/~aeb/linux/kbd/scancodes.html
//   https://wiki.osdev.org/%228042%22_PS/2_Controller
//   https://wiki.osdev.org/PS/2_Keyboard
//   https://wiki.osdev.org/PS/2_Mouse
//   https://wiki.osdev.org/Mouse_Input
//   http://www.s100computers.com/My%20System%20Pages/MSDOS%20Board/PC%20Keyboard.pdf

pub use self::{
    controller::Controller,
    keyboard::{Keyboard, KeyboardType},
    mouse::{Mouse, MouseType},
};

mod controller;
mod keyboard;
mod mouse;

pub mod error;
pub mod flags;

const COMMAND_ACKNOWLEDGED: u8 = 0xfa;
const SELF_TEST_PASSED: u8 = 0xaa;
const SELF_TEST_FAILED: u8 = 0xfc;
const RESEND: u8 = 0xfe;
