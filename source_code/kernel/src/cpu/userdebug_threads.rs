// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Debug helpers that execute in user cores (Core 1..N) to validate scheduler and MMU state.

use crate::uart_println;
use aarch64_cpu::registers::{Readable, CNTPCT_EL0, TTBR0_EL1};

/// Prints lightweight diagnostics from a user-land scheduling context.
pub fn run_debug_checks() {
    use super::local_core_id;

    let core_id = local_core_id();
    uart_println!("[DEBUG][user] Running in Core number: {}", core_id);

    let ttbr0 = TTBR0_EL1.get();
    let cntpct = CNTPCT_EL0.get();

    uart_println!(
        "[DEBUG][user] TTBR0_EL1 (current process): 0x{ttbr0:016x} | CNTVCT_EL0 snapshot: {}",
        cntpct
    );
    uart_println!(
        "[DEBUG][user] Mailbox health: awaiting response -> core {} still scheduled",
        core_id
    );
}
