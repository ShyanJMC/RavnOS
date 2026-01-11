// SPDX-License-Identifier: MIT OR Apache-2.0
//
//! Delegates driver initialization to the board-specific implementations.

use super::dtb::Summary as DtbSummary;
use crate::bsp::drivers_interface::{self, BoardDriver};
#[cfg(feature = "bsp_rpi4")]
use crate::bsp::raspberrypi4b;
#[cfg(feature = "bsp_rpi5")]
use crate::bsp::raspberrypi5;
use core::sync::atomic::{AtomicBool, Ordering};

/// Initialize the active board driver based on DTB information.
pub fn init(summary: &DtbSummary) -> Result<(), &'static str> {
    static INIT_DONE: AtomicBool = AtomicBool::new(false);
    if INIT_DONE.swap(true, Ordering::SeqCst) {
        return Err("BSP drivers already initialized");
    }

    let mut driver: Option<&'static dyn BoardDriver> = None;

    #[cfg(feature = "bsp_rpi4")]
    {
        let candidate: &'static dyn BoardDriver = &raspberrypi4b::drivers::BOARD;
        if candidate.matches(summary) {
            driver = Some(candidate);
        }
    }

    #[cfg(feature = "bsp_rpi5")]
    {
        let candidate: &'static dyn BoardDriver = &raspberrypi5::drivers::BOARD;
        if driver.is_none() && candidate.matches(summary) {
            driver = Some(candidate);
        }
    }

    let driver = driver.ok_or("Unsupported DTB: no matching board driver registered")?;

    unsafe {
        driver.init(summary)?;
    }

    drivers_interface::set_active_driver(driver);

    Ok(())
}
