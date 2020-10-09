#![no_std]

// References:
//   https://web.archive.org/web/20091124055529/http://www.computer-engineering.org/index.php?title=Main_Page
//   https://web.archive.org/web/20200616182211/https://www.win.tue.nl/~aeb/linux/kbd/scancodes.html
//   https://wiki.osdev.org/%228042%22_PS/2_Controller
//   https://wiki.osdev.org/PS2_Keyboard
//   https://wiki.osdev.org/PS/2_Mouse
//   https://wiki.osdev.org/Mouse_Input
//   http://www.s100computers.com/My%20System%20Pages/MSDOS%20Board/PC%20Keyboard.pdf

pub use self::{
    controller::Controller,
    keyboard::{Keyboard, KeyboardType},
};

mod controller;
mod keyboard;

pub mod error;
pub mod flags;
