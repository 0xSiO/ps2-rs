#![no_std]

// References:
//   https://www.avrfreaks.net/sites/default/files/PS2%20Keyboard.pdf
//   http://www.s100computers.com/My%20System%20Pages/MSDOS%20Board/PC%20Keyboard.pdf

mod controller;
mod flags;
mod response;

pub use self::{controller::Controller, flags::*, response::*};
