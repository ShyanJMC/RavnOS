// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Secondary-core bring-up for QEMU. Two modes are supported:
// - Default (`-M raspi4b`): reuse the Raspberry Pi mailbox spin-table path.
// - `--features qemu_psci`: use PSCI CPU_ON calls for QEMU's `virt` machine.

#[cfg(feature = "qemu_psci")]
mod imp {
    use crate::uart_println;
    use core::arch::asm;

    pub const DEFAULT_CORE_COUNT: usize = 4;

    /// Used by `arch` code to find the early boot core.
    #[no_mangle]
    #[link_section = ".text._start_arguments"]
    pub static BOOT_CORE_ID: u64 = 0;

    const PSCI_CPU_ON: u32 = 0x8400_0003;

    extern "C" {
        static __rpi_phys_binary_load_addr: u8;
    }

    #[inline(always)]
    unsafe fn psci_call(function_id: u32, arg0: u64, arg1: u64, arg2: u64) -> u64 {
        let mut x0 = function_id as u64;
        asm!(
            "smc #0",
            inout("x0") x0,
            in("x1") arg0,
            in("x2") arg1,
            in("x3") arg2,
            options(nostack, preserves_flags)
        );
        x0
    }

    unsafe fn psci_cpu_on(target_mpidr: u64, entry: u64) -> Result<(), i32> {
        match psci_call(PSCI_CPU_ON, target_mpidr, entry, 0) {
            0 => Ok(()),
            err => Err(err as i32),
        }
    }

    /// Ask the PSCI firmware to boot a secondary core when emulating a generic `virt` machine.
    pub fn start_secondary_core(core_id: usize) {
        let entry = unsafe { (&__rpi_phys_binary_load_addr as *const u8) as u64 };

        if core_id == 0 {
            uart_println!("[0] PSCI: boot core 0 already active; skipping CPU_ON");
            return;
        }

        match unsafe { psci_cpu_on(core_id as u64, entry) } {
            Ok(()) => uart_println!(
                "[0] PSCI: requested start of core {} at entry {:#x}",
                core_id,
                entry
            ),
            Err(code) => uart_println!(
                "[0] PSCI: failed to start core {} (error {})",
                core_id,
                code
            ),
        }
    }
}

#[cfg(not(feature = "qemu_psci"))]
mod imp {
    use crate::bsp::raspberrypi::cpu as rpi_cpu;
    use crate::uart_println;

    pub const DEFAULT_CORE_COUNT: usize = 4;

    /// Used by `arch` code to find the early boot core.
    #[no_mangle]
    #[link_section = ".text._start_arguments"]
    pub static BOOT_CORE_ID: u64 = 0;

    /// Route secondary-core requests through the Raspberry Pi spin-table path, which QEMU's
    /// `-M raspi4b` emulates closely enough for testing.
    pub fn start_secondary_core(core_id: usize) {
        uart_println!(
            "[0] QEMU raspi4b: forwarding secondary-core start request for core {}",
            core_id
        );
        rpi_cpu::start_secondary_core(core_id);
    }
}

pub use imp::{start_secondary_core, BOOT_CORE_ID, DEFAULT_CORE_COUNT};
