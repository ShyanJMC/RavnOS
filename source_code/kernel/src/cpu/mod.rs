mod boot;
pub mod kernel_threads;
pub mod process;
pub mod scheduler;
pub mod userdebug_threads;

use aarch64_cpu::asm;
use aarch64_cpu::registers::{Readable, MPIDR_EL1};
use core::arch::asm as core_asm;

extern "C" {
    fn el2_to_el1();
}

/// Pause execution on the current core indefinitely.
#[inline(always)]
pub fn wait_forever() -> ! {
    loop {
        asm::wfe()
    }
}

/// Returns the ID of the currently executing core.
pub fn local_core_id() -> u8 {
    (MPIDR_EL1.get() & 0b11) as u8
}

/// Ensure that the CPU is executing at EL1 by dropping out of EL2 when required.
pub fn ensure_el1() {
    let mut current_el: u64 = 0;
    unsafe {
        core_asm!(
            "mrs {0}, CurrentEL",
            out(reg) current_el,
            options(nomem, nostack, preserves_flags)
        );
    }
    let level = (current_el >> 2) & 0b11;
    if level == 2 {
        unsafe {
            el2_to_el1();
        }
    }
}
