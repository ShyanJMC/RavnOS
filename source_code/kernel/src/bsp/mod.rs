// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

//! Board Support Package facade.

#[cfg(any(feature = "bsp_rpi4", feature = "bsp_rpi5", feature = "bsp_qemu"))]
use crate::uart_println;
#[cfg(any(feature = "bsp_rpi4", feature = "bsp_rpi5", feature = "bsp_qemu"))]
use core::mem::MaybeUninit;

pub mod drivers;
pub mod drivers_interface;
#[cfg(feature = "bsp_qemu")]
pub mod qemu;
#[cfg(any(feature = "bsp_rpi4", feature = "bsp_qemu"))]
pub mod raspberrypi4b;
#[cfg(feature = "bsp_rpi5")]
pub mod raspberrypi5;

#[cfg(any(
    all(feature = "bsp_rpi4", feature = "bsp_rpi5"),
    all(feature = "bsp_rpi4", feature = "bsp_qemu"),
    all(feature = "bsp_rpi5", feature = "bsp_qemu")
))]
compile_error!("Select only one BSP feature at a time.");

#[cfg(not(any(feature = "bsp_rpi4", feature = "bsp_rpi5", feature = "bsp_qemu")))]
compile_error!("Enable a BSP feature such as `bsp_rpi4`, `bsp_rpi5`, or `bsp_qemu`.");

#[cfg(any(feature = "bsp_rpi4", feature = "bsp_rpi5", feature = "bsp_qemu"))]
mod raspberrypi;

#[cfg(any(feature = "bsp_rpi4", feature = "bsp_rpi5", feature = "bsp_qemu"))]
pub use raspberrypi::dtb::Summary as DtbSummary;

/// Bring up board drivers and log them.
#[cfg(any(feature = "bsp_rpi4", feature = "bsp_rpi5", feature = "bsp_qemu"))]
pub fn init() -> Result<(), &'static str> {
    let mut fallback_summary = MaybeUninit::<raspberrypi::dtb::Summary>::uninit();
    let (dtb_summary, dtb_loaded) = match raspberrypi::dtb::ensure_loaded() {
        Ok(summary) => (summary, true),
        Err(_) => {
            fallback_summary.write(raspberrypi::dtb::Summary::fallback());
            (unsafe { fallback_summary.assume_init_ref() }, false)
        }
    };

    raspberrypi::driver::init(dtb_summary)?;

    if !dtb_loaded {
        uart_println!(
            "[0] WARNING: DTB missing at {:#x}; using fallback peripheral layout",
            raspberrypi::dtb::load_addr()
        );
    }

    Ok(())
}

#[cfg(any(feature = "bsp_rpi4", feature = "bsp_rpi5", feature = "bsp_qemu"))]
pub fn probe_dtb() -> Option<DtbSummary> {
    raspberrypi::dtb::probe()
}

#[cfg(any(feature = "bsp_rpi4", feature = "bsp_rpi5"))]
pub fn start_secondary_core(core_id: usize) {
    raspberrypi::cpu::start_secondary_core(core_id);
}

#[cfg(feature = "bsp_qemu")]
pub fn start_secondary_core(core_id: usize) {
    qemu::cpu::start_secondary_core(core_id);
}

/// Name of the active board reported by the driver subsystem.
pub fn board_name() -> &'static str {
    drivers_interface::board_name()
}

/// Default number of cores when the DTB omits this information.
pub fn default_core_count() -> usize {
    drivers_interface::default_core_count()
}
