// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2023 Andre Richter <andre.o.richter@gmail.com>

//! BSP Processor code.

use core::ptr::write_volatile;

use crate::await_kernel_uart_println;
use crate::bsp::raspberrypi::dtb;

use aarch64_cpu::asm::barrier::{dsb, isb, SY};
use aarch64_cpu::asm::sev;

/// Hint for how many cores are expected to be present on the Raspberry Pi 4.
pub const DEFAULT_CORE_COUNT: usize = 4;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Used by `arch` code to find the early boot core.
#[cfg(any(feature = "bsp_rpi4", feature = "bsp_rpi5"))]
#[no_mangle]
#[link_section = ".text._start_arguments"]
pub static BOOT_CORE_ID: u64 = 0;

extern "C" {
    static __rpi_phys_binary_load_addr: u8;
}

const LEGACY_SPIN_TABLE_BASE: u64 = 0x4000_0000;
const LEGACY_SPIN_TABLE_STRIDE: u64 = 0x10;
const LEGACY_RELEASE_OFFSET: u64 = 0x8;

enum ReleaseSlot {
    /// Standard spin-table entry advertised through `cpu-release-addr`.
    SpinTable(u64),
    /// Fallback to the legacy mailbox layout used by early cores/QEMU builds.
    LegacyMailbox { entry_addr: u64, release_addr: u64 },
}

/// Start a single secondary core by poking the Raspberry Pi mailbox.
pub fn start_secondary_core(core_id: usize) {
    if core_id == 0 {
        await_kernel_uart_println!(
            "[0] Core 0 is already running kernel_init(); skipping mailbox poke"
        );
        return;
    }

    let entry = unsafe { (&__rpi_phys_binary_load_addr as *const u8) as u64 };

    match release_slot_for(core_id) {
        Some(ReleaseSlot::SpinTable(slot_addr)) => {
            await_kernel_uart_println!(
                "[0] Spin-table release for core {}: slot {:#x}, entry {:#x}",
                core_id,
                slot_addr,
                entry
            );

            unsafe {
                write_volatile(slot_addr as *mut u64, entry);
                dsb(SY);
                sev();
            }

            await_kernel_uart_println!("[0] Core {} release slot updated", core_id);
        }
        Some(ReleaseSlot::LegacyMailbox {
            entry_addr,
            release_addr,
        }) => {
            await_kernel_uart_println!(
                "[0] Legacy mailbox path for core {}: entry {:#x}",
                core_id,
                entry_addr
            );
            await_kernel_uart_println!(
                "[0] Setting legacy spin-table address for core {} to {:#x}",
                core_id,
                entry
            );

            unsafe {
                write_volatile(entry_addr as *mut u64, entry);
                dsb(SY);
                isb(SY);
                write_volatile(release_addr as *mut u64, 0);
                dsb(SY);
                sev();
            }

            await_kernel_uart_println!("[0] Core {} started via legacy mailbox", core_id);
        }
        None => {
            await_kernel_uart_println!(
                "[0] ERROR: no spin-table or mailbox slot known for core {}",
                core_id
            );
        }
    }
}

fn release_slot_for(core_id: usize) -> Option<ReleaseSlot> {
    if let Some(slots) = dtb::cpu_release_addrs() {
        if let Some(&addr) = slots.get(core_id) {
            if addr != 0 {
                return Some(ReleaseSlot::SpinTable(addr));
            }
        }
    }

    if core_id == 0 {
        None
    } else {
        let entry_addr = LEGACY_SPIN_TABLE_BASE + (core_id as u64) * LEGACY_SPIN_TABLE_STRIDE;
        Some(ReleaseSlot::LegacyMailbox {
            entry_addr,
            release_addr: entry_addr + LEGACY_RELEASE_OFFSET,
        })
    }
}
