// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Shared interfaces for board-specific driver implementations.

use crate::{
    console,
    synchronization::{interface::Mutex, NullLock},
};
use core::fmt;

type DriverSummary = super::raspberrypi::dtb::Summary;

/// Common operations every board-specific driver bundle must implement.
pub trait BoardDriver: Send + Sync {
    /// Whether the driver matches the hardware described by the DTB summary.
    fn matches(&self, summary: &DriverSummary) -> bool;

    /// Initialize all drivers needed by the board (UART for now).
    ///
    /// # Safety
    ///
    /// Drivers touch MMIO ranges and therefore must only be initialized once.
    unsafe fn init(&'static self, summary: &DriverSummary) -> Result<(), &'static str>;

    /// Human readable board name.
    fn board_name(&self) -> &'static str;

    /// Default number of cores expected on the board.
    fn default_core_count(&self) -> usize;

    /// UART implementation used for logging.
    fn uart(&self) -> &'static dyn console::interface::All;
}

/// Keeps track of the active board driver so that logging helpers can reach it.
static ACTIVE_DRIVER: NullLock<Option<&'static dyn BoardDriver>> = NullLock::new(None);

/// Register the board driver once initialization succeeds.
pub fn set_active_driver(driver: &'static dyn BoardDriver) {
    ACTIVE_DRIVER.lock(|slot| *slot = Some(driver));
}

/// Return the currently selected board driver, if any.
pub fn active_driver() -> Option<&'static dyn BoardDriver> {
    ACTIVE_DRIVER.lock(|slot| *slot)
}

/// Helper used by the panic/logging macros.
pub fn write_uart(args: fmt::Arguments) {
    if let Some(driver) = active_driver() {
        let uart = driver.uart();
        let _ = uart.write_fmt(args);
        uart.flush();
    }
}

/// Obtain the name of the selected board.
pub fn board_name() -> &'static str {
    active_driver()
        .map(|driver| driver.board_name())
        .unwrap_or("Unknown SBC")
}

/// Core-count hint published by the active board driver.
pub fn default_core_count() -> usize {
    active_driver()
        .map(|driver| driver.default_core_count())
        .unwrap_or(1)
}

#[macro_export]
macro_rules! uart_print {
    ($($arg:tt)*) => {{
        $crate::bsp::drivers_interface::write_uart(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! uart_println {
    () => ({
        $crate::bsp::drivers_interface::write_uart(format_args!("\n"));
    });
    ($($arg:tt)*) => ({
        $crate::bsp::drivers_interface::write_uart(format_args_nl!($($arg)*));
    });
}
