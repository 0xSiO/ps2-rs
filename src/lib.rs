#![no_std]
#![feature(const_mut_refs)]
//! This crate provides comprehensive low-level access to the PS/2 controller and PS/2 devices. It
//! uses a poll-based approach with a timeout to read and write data to the IO ports.
//!
//! # Example
//! The below example implements the initialization process outlined on the [OSDev wiki](https://web.archive.org/web/20201112021519/https://wiki.osdev.org/%228042%22_PS/2_Controller#Initialising_the_PS.2F2_Controller).
//! We skip steps 1 and 2 by assuming the PS/2 controller exists and is initialized.
//! ```no_run
//! use ps2::{Controller, error::ControllerError, flags::ControllerConfigFlags};
//!
//! fn initialize() -> Result<(), ControllerError> {
//!     let mut controller = unsafe { Controller::new() };
//!
//!     // Step 3: Disable devices
//!     controller.disable_keyboard()?;
//!     controller.disable_mouse()?;
//!
//!     // Step 4: Flush data buffer
//!     let _ = controller.read_data();
//!
//!     // Step 5: Set config
//!     let mut config = controller.read_config()?;
//!     // Disable interrupts and scancode translation
//!     config.set(
//!         ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT
//!             | ControllerConfigFlags::ENABLE_MOUSE_INTERRUPT
//!             | ControllerConfigFlags::ENABLE_TRANSLATE,
//!         false,
//!     );
//!     controller.write_config(config)?;
//!
//!     // Step 6: Controller self-test
//!     controller.test_controller()?;
//!     // Write config again in case of controller reset
//!     controller.write_config(config)?;
//!
//!     // Step 7: Determine if there are 2 devices
//!     let has_mouse = if config.contains(ControllerConfigFlags::DISABLE_MOUSE) {
//!         controller.enable_mouse()?;
//!         config = controller.read_config()?;
//!         // If mouse is working, this should now be unset
//!         !config.contains(ControllerConfigFlags::DISABLE_MOUSE)
//!     } else {
//!         false
//!     };
//!     // Disable mouse. If there's no mouse, this is ignored
//!     controller.disable_mouse()?;
//!
//!     // Step 8: Interface tests
//!     let keyboard_works = controller.test_keyboard().is_ok();
//!     let mouse_works = has_mouse && controller.test_mouse().is_ok();
//!
//!     // Step 9: Enable devices
//!     config = controller.read_config()?;
//!     if keyboard_works {
//!         controller.enable_keyboard()?;
//!         config.set(ControllerConfigFlags::DISABLE_KEYBOARD, false);
//!         config.set(ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT, true);
//!     }
//!     if mouse_works {
//!         controller.enable_mouse()?;
//!         config.set(ControllerConfigFlags::DISABLE_MOUSE, false);
//!         config.set(ControllerConfigFlags::ENABLE_MOUSE_INTERRUPT, true);
//!     }
//!
//!     // Step 10: Reset devices
//!     controller.keyboard().reset_and_self_test().unwrap();
//!     controller.mouse().reset_and_self_test().unwrap();
//!
//!     // This will start streaming events from the mouse
//!     controller.mouse().enable_data_reporting().unwrap();
//!
//!     // Write last configuration to enable devices and interrupts
//!     controller.write_config(config)?;
//!
//!     Ok(())
//! }
//! ```
//!
// TODO: Docs on actually reading data from the devices

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
