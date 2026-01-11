mod boot;
pub mod kernel_threads;
pub mod scheduler;
pub mod userdebug_threads;

use aarch64_cpu::asm;
use aarch64_cpu::registers::{Readable, MPIDR_EL1};

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
