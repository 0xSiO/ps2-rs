#![no_std]

// References:
//   https://wiki.osdev.org/%228042%22_PS/2_Controller
//   https://wiki.osdev.org/PS2_Keyboard
//   https://wiki.osdev.org/PS/2_Mouse
//   https://wiki.osdev.org/Mouse_Input
//   https://www.avrfreaks.net/sites/default/files/PS2%20Keyboard.pdf
//   http://www.s100computers.com/My%20System%20Pages/MSDOS%20Board/PC%20Keyboard.pdf

mod controller;
pub mod error;
pub mod flags;
mod response;

pub use self::controller::Controller;
