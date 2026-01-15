// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Debug helpers that must always execute on Core 0 (kernel domain).

use crate::await_kernel_uart_println;
use crate::bsp;
use crate::cpu::{local_core_id, process, scheduler};
use aarch64_cpu::registers::{
    Readable, CNTKCTL_EL1, CNTP_CTL_EL0, CNTP_TVAL_EL0, DAIF, MAIR_EL1, SCTLR_EL1, TTBR0_EL1,
    TTBR1_EL1,
};
use core::sync::atomic::{AtomicU32, Ordering};

static KERNEL_DEBUG_PRINTED_SLOTS: AtomicU32 = AtomicU32::new(0);
const ENABLE_KERNEL_DEBUG_GUARD: bool =
    false; // Pon en true si quieres silenciar los prints después de la primera iteración por slot.

/// Prints scheduler/memory health information while running in the kernel core.
pub fn run_debug_checks() -> () {
    let core_id = local_core_id();
    let slot = scheduler::current_task_slot(core_id as usize);

    if ENABLE_KERNEL_DEBUG_GUARD {
        if !reserve_kernel_slot_once(slot) {
            return;
        }
    }
    let current_el: u64;
    unsafe {
        core::arch::asm!("mrs {0}, CurrentEL", out(reg) current_el, options(nomem, nostack, preserves_flags));
    }
    let current_el = (current_el >> 2) & 0b11;
    await_kernel_uart_println!(
        "[DEBUG][kernel] Running in Core number: {} (CurrentEL={})",
        core_id,
        current_el
    );

    // Dump basic MMU state for quick validation against the design in mmu-scheduler.ai.
    let ttbr0 = TTBR0_EL1.get();
    let ttbr1 = TTBR1_EL1.get();
    let sctlr = SCTLR_EL1.get();
    let mair = MAIR_EL1.get();

    await_kernel_uart_println!(
        "[DEBUG][kernel] TTBR0_EL1 (user tables): 0x{ttbr0:016x} | TTBR1_EL1 (kernel tables): 0x{ttbr1:016x}"
    );
    await_kernel_uart_println!(
        "[DEBUG][kernel] SCTLR_EL1: 0x{sctlr:016x} (MMU {}, caches {})",
        if sctlr & 1 != 0 { "ON" } else { "OFF" },
        if sctlr & (1 << 2) != 0 { "ON" } else { "OFF" }
    );
    await_kernel_uart_println!("[DEBUG][kernel] MAIR_EL1 attr table: 0x{mair:016x}");
    await_kernel_uart_println!(
        "[DEBUG][kernel] Scheduler quanta: 5ms | Core claim flags verified on Core {}",
        core_id
    );

    let cntp_ctl = CNTP_CTL_EL0.get();
    let cntp_tval = CNTP_TVAL_EL0.get();
    let daif = DAIF.get();
    let cntkctl = CNTKCTL_EL1.get();
    let irq_masked = (daif & (1 << 7)) != 0;
    let enable = (cntp_ctl & 0b1) != 0;
    let mask = (cntp_ctl & 0b10) != 0;
    let pending = (cntp_ctl & 0b100) != 0;
    await_kernel_uart_println!(
        "[DEBUG][kernel] CNTP_CTL_EL0: enable={} mask={} pending={} | TVAL {} | CNTKCTL_EL1=0x{:04x} | DAIF 0x{:04x}",
        enable,
        mask,
        pending,
        cntp_tval,
        cntkctl,
        daif
    );

    if irq_masked {
        await_kernel_uart_println!(
            "[DEBUG][kernel] DAIF.I set; re-enabling IRQs via scheduler::enable_irq()"
        );
        unsafe { scheduler::enable_irq() };
    }

    if let Some(timer_irq) = bsp::timer_irq_snapshot() {
        await_kernel_uart_println!(
            "[DEBUG][kernel] GIC timer snapshot: pending={} enabled={} active={} | GICC_CTLR=0x{:02x} PMR=0x{:02x}",
            timer_irq.pending,
            timer_irq.enabled,
            timer_irq.active,
            timer_irq.cpu_ctlr,
            timer_irq.cpu_pmr
        );
    } else {
        await_kernel_uart_println!("[DEBUG][kernel] Timer IRQ snapshot unavailable");
    }

    bsp::log_timer_irq_state("kernel");

    let heartbeat = scheduler::read_irq_heartbeat();
    if heartbeat != 0 {
        await_kernel_uart_println!(
            "[DEBUG][kernel] IRQ heartbeat last CNTPCT value: {}",
            heartbeat
        );
    } else {
        await_kernel_uart_println!("[DEBUG][kernel] IRQ heartbeat still zero");
    }

    if core_id == 0 {
        let _ = process::with_kernel_process(slot, |pcb| {
            await_kernel_uart_println!(
                "[DEBUG][kernel] PCB pid {} state {:?} priority {}",
                pcb.pid,
                pcb.state,
                pcb.priority
            );
        });
    }

    scheduler::log_scheduler_snapshot("[kernel-debug]", core_id as usize);
    return;
}

fn reserve_kernel_slot_once(slot: usize) -> bool {
    let max_slots = scheduler::MAX_KERNEL_TASKS.min(32);
    if slot >= max_slots {
        return false;
    }
    let mask = 1u32 << slot;
    (KERNEL_DEBUG_PRINTED_SLOTS.fetch_or(mask, Ordering::Relaxed) & mask) == 0
}
