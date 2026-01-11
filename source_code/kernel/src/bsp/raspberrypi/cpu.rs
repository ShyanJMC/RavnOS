// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

//! BSP Processor code.

use core::ptr::write_volatile;

use crate::uart_println;

use aarch64_cpu::asm::barrier::{dsb, isb, SY};
use aarch64_cpu::asm::sev;

/// Hint for how many cores are expected to be present on the Raspberry Pi 4.
pub const DEFAULT_CORE_COUNT: usize = 4;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Used by `arch` code to find the early boot core.
#[no_mangle]
#[link_section = ".text._start_arguments"]
pub static BOOT_CORE_ID: u64 = 0;

/// Start secondary cores by poking the Raspberry Pi mailbox.
pub fn start_secondary_cores(core_count: usize) {
    const CORE_START_ADDR: u64 = 0x80000;
    const SPIN_TABLE_BASE: u64 = 0x4000_0000;
    const SPIN_TABLE_STRIDE: u64 = 0x10;
    const RELEASE_OFFSET: u64 = 0x8;

    for core in 1..core_count {
        let entry_addr = SPIN_TABLE_BASE + (core as u64) * SPIN_TABLE_STRIDE;
        let release_addr = entry_addr + RELEASE_OFFSET;

        uart_println!(
            "[0] Starting core {} with total MAILBOX; {}",
            core,
            entry_addr
        );
        uart_println!(
            "[0] Setting Spin Table for core {} with address {}",
            core,
            CORE_START_ADDR
        );

        unsafe {
            write_volatile(entry_addr as *mut u64, CORE_START_ADDR);
            dsb(SY);
            isb(SY);
            write_volatile(release_addr as *mut u64, 0);
            dsb(SY);
            sev();
        }

        uart_println!("[0] Core {} started", core);
    }
}
