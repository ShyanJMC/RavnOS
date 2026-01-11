// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

//! BSP console facilities.

use crate::{bsp::drivers_interface, console};

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Return a reference to the console.
pub fn console() -> &'static dyn console::interface::All {
    drivers_interface::active_driver()
        .map(|driver| driver.uart())
        .expect("UART console requested before initialization")
}
