#![no_std]
#![feature(const_mut_refs)]
#![warn(rust_2018_idioms)]
//! This crate provides comprehensive low-level access to the PS/2 controller and PS/2 devices. It
//! uses a poll-based approach with a timeout to read and write data to the IO ports.
//!
//! # Examples
//!
//! The below example implements the initialization process outlined on the [OSDev wiki](https://web.archive.org/web/20201112021519/https://wiki.osdev.org/%228042%22_PS/2_Controller#Initialising_the_PS.2F2_Controller).
//! We skip steps 1 and 2 by assuming the PS/2 controller exists and is supported on the current
//! hardware.
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
//!     // Step 9 - 10: Enable and reset devices
//!     config = controller.read_config()?;
//!     if keyboard_works {
//!         controller.enable_keyboard()?;
//!         config.set(ControllerConfigFlags::DISABLE_KEYBOARD, false);
//!         config.set(ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT, true);
//!         controller.keyboard().reset_and_self_test().unwrap();
//!     }
//!     if mouse_works {
//!         controller.enable_mouse()?;
//!         config.set(ControllerConfigFlags::DISABLE_MOUSE, false);
//!         config.set(ControllerConfigFlags::ENABLE_MOUSE_INTERRUPT, true);
//!         controller.mouse().reset_and_self_test().unwrap();
//!         // This will start streaming events from the mouse
//!         controller.mouse().enable_data_reporting().unwrap();
//!     }
//!
//!     // Write last configuration to enable devices and interrupts
//!     controller.write_config(config)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! Once the controller is initialized and the devices are working properly, they will place input
//! in the data buffer at IO port `0x60`. You can read from this buffer at any time using
//! [`Controller::read_data`]. If you plan on using a poll-based approach to handle device input,
//! be aware that either device may write data to this buffer at any time, and as far as I know
//! there is no way to tell which bytes come from which device.
//!
//! A much better way of handling input is to use interrupts: define handlers for IRQ1 (keyboard)
//! and IRQ12 (mouse) and read the data then. You can use [`Controller::read_data`] for both
//! keyboard and mouse data, or you can use [`Mouse::read_data_packet`] which is a convenient
//! wrapper around [`Controller::read_data`] for mouse packets.
//!
//! # Further Reading
//!
//! Below are some resources I used to develop this library. Note that some resources describing
//! the PS/2 protocol conflict with each other, so this library is my best effort at testing and
//! verifying the accuracy of these resources. If you find that something is missing or doesn't
//! seem quite right, feel free to open an [issue](https://github.com/lucis-fluxum/ps2-rs/issues/new).
//!
//! - [Adam Chapweske's old site][chepweske], which has several detailed write-ups.
//! - [Andries Brouwer's "Keyboard scancodes"][brouwer], which is about much more than just
//!   scancodes.
//! - The OSDev wiki's pages on the [PS/2 controller][osdev_ps2], [keyboard][osdev_keyboard],
//!   [mouse][osdev_mouse], and [mouse input][osdev_mouse_input].
//! - [This PDF][mike_2009] from what appears to be an operating systems development course. I found
//!   the language very approachable and helpful for learning a lot of the relevant terminology.
//! - [This summary][netcore2k_8042] of the PS/2 registers and commands. See [here][netcore2k_keyboard] for keyboard commands.
//!
//! [chepweske]: https://web.archive.org/web/20091124055529/http://www.computer-engineering.org/index.php?title=Main_Page
//! [brouwer]: https://web.archive.org/web/20200616182211/https://www.win.tue.nl/~aeb/linux/kbd/scancodes.html
//! [osdev_ps2]: https://web.archive.org/web/20201112021519/https://wiki.osdev.org/%228042%22_PS/2_Controller
//! [osdev_keyboard]: https://web.archive.org/web/20201112022746/https://wiki.osdev.org/PS/2_Keyboard
//! [osdev_mouse]: https://web.archive.org/web/20200930160800/https://wiki.osdev.org/PS/2_Mouse
//! [osdev_mouse_input]: https://web.archive.org/web/20201112014603/https://wiki.osdev.org/Mouse_Input
//! [mike_2009]: http://www.s100computers.com/My%20System%20Pages/MSDOS%20Board/PC%20Keyboard.pdf
//! [netcore2k_8042]: https://web.archive.org/web/20201023081327/http://helppc.netcore2k.net/hardware/8042
//! [netcore2k_keyboard]: https://web.archive.org/web/20201023082815/http://helppc.netcore2k.net/hardware/keyboard-commands

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
