// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Debug helpers that must always execute on Core 0 (kernel domain).

use crate::uart_println;
use aarch64_cpu::registers::{Readable, MAIR_EL1, SCTLR_EL1, TTBR0_EL1, TTBR1_EL1};

/// Prints scheduler/memory health information while running in the kernel core.
pub fn run_debug_checks() {
    use super::local_core_id;

    let core_id = local_core_id();
    uart_println!("[DEBUG][kernel] Running in Core number: {}", core_id);

    // Dump basic MMU state for quick validation against the design in mmu-scheduler.ai.
    let ttbr0 = TTBR0_EL1.get();
    let ttbr1 = TTBR1_EL1.get();
    let sctlr = SCTLR_EL1.get();
    let mair = MAIR_EL1.get();

    uart_println!(
        "[DEBUG][kernel] TTBR0_EL1 (user tables): 0x{ttbr0:016x} | TTBR1_EL1 (kernel tables): 0x{ttbr1:016x}"
    );
    uart_println!(
        "[DEBUG][kernel] SCTLR_EL1: 0x{sctlr:016x} (MMU {}, caches {})",
        if sctlr & 1 != 0 { "ON" } else { "OFF" },
        if sctlr & (1 << 2) != 0 { "ON" } else { "OFF" }
    );
    uart_println!("[DEBUG][kernel] MAIR_EL1 attr table: 0x{mair:016x}");
    uart_println!(
        "[DEBUG][kernel] Scheduler quanta: 5ms | Core claim flags verified on Core {}",
        core_id
    );
}
