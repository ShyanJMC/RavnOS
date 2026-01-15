// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Debug helpers that execute in user cores (Core 1..N) to validate scheduler and MMU state.

use crate::await_kernel_uart_println;
use crate::bsp;
use crate::cpu::process;
use aarch64_cpu::registers::{Readable, CNTPCT_EL0, TPIDR_EL0};
use core::sync::atomic::{AtomicU32, Ordering};

static USER_DEBUG_PRINTED_SLOTS: AtomicU32 = AtomicU32::new(0);
const ENABLE_USER_DEBUG_GUARD: bool =
    false; // Pon en true si quieres volver a desactivar los prints repetidos por slot.

/// Prints lightweight diagnostics from a user-land scheduling context.
pub fn run_debug_checks() -> () {
    let (core_id, slot) = thread_identity();

    if ENABLE_USER_DEBUG_GUARD {
        if !reserve_user_slot_once(slot) {
            return;
        }
    }

    await_kernel_uart_println!(
        "[DEBUG][user] Running in Core number: {} | slot {}",
        core_id,
        slot
    );

    let cntpct = CNTPCT_EL0.get();

    await_kernel_uart_println!(
        "[DEBUG][user] CNTPCT_EL0 snapshot: {} | Mailbox health: awaiting response -> core {} still scheduled",
        cntpct,
        core_id
    );

    if let Some(snapshot) = bsp::timer_irq_snapshot() {
        await_kernel_uart_println!(
            "[DEBUG][user] GIC timer snapshot: pending={} enabled={} active={} | GICC_CTLR=0x{:02x} PMR=0x{:02x}",
            snapshot.pending,
            snapshot.enabled,
            snapshot.active,
            snapshot.cpu_ctlr,
            snapshot.cpu_pmr
        );
    } else {
        await_kernel_uart_println!("[DEBUG][user] Timer IRQ snapshot unavailable");
    }

    let _ = process::with_user_process(slot, |pcb| {
        await_kernel_uart_println!(
            "[DEBUG][user] PCB pid {} state {:?} ttbr0 0x{:016x}",
            pcb.pid,
            pcb.state,
            pcb.ttbr0
        );
    });

    crate::cpu::scheduler::log_scheduler_snapshot("[user-debug]", core_id as usize);
    return;
}

fn thread_identity() -> (u8, usize) {
    let raw = TPIDR_EL0.get();
    let core = ((raw >> 32) & 0xff) as u8;
    let slot = (raw & 0xffff_ffff) as usize;
    (core, slot)
}

fn reserve_user_slot_once(slot: usize) -> bool {
    let max_slots = process::USER_MAX_PROCESSES.min(32);
    if slot >= max_slots {
        return false;
    }
    let mask = 1u32 << slot;
    (USER_DEBUG_PRINTED_SLOTS.fetch_or(mask, Ordering::Relaxed) & mask) == 0
}
