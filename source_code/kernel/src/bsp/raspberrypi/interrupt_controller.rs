// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Minimal GIC-400 bring-up so the per-core generic timer non-secure physical interrupt (PPI #14 ->
// INTID 30) can raise IRQs.

use core::convert::TryInto;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, Ordering};

use aarch64_cpu::asm::barrier::{dsb, isb, SY};
use aarch64_cpu::registers::{CNTP_CTL_EL0, CNTP_CVAL_EL0, CNTP_TVAL_EL0, CNTPCT_EL0, MPIDR_EL1};
use tock_registers::interfaces::{Readable, Writeable};

use crate::await_kernel_uart_println;
use crate::bsp::raspberrypi::dtb;
use crate::cpu;

/// Generic timer non-secure physical interrupt (PPI #14 -> INTID 30 = 16 + 14).
const TIMER_PPI_ID: u32 = 30;

const GICD_CTLR: usize = 0x0000;
const GICD_IGROUPR: usize = 0x0080;
const GICD_ISENABLER: usize = 0x0100;
const GICD_IPRIORITYR: usize = 0x0400;
const GICD_ISPENDR: usize = 0x0200;
const GICD_ICPENDR: usize = 0x0280;
const GICD_ISACTIVER: usize = 0x0300;
const GICD_ICFGR: usize = 0x0C00;

const GICC_CTLR: usize = 0x0000;
const GICC_PMR: usize = 0x0004;
const GICC_BPR: usize = 0x0008;
const GICC_IAR: usize = 0x000C;
const GICC_EOIR: usize = 0x0010;
const SPURIOUS_IRQ_ID: u32 = 0x03ff;

#[derive(Copy, Clone)]
struct GicState {
    dist_base: usize,
    cpu_base: usize,
}

static mut GIC_STATE: Option<GicState> = None;
static DISTRIBUTOR_ENABLED: AtomicBool = AtomicBool::new(false);

#[derive(Copy, Clone, Debug)]
pub struct TimerIrqState {
    pub pending: bool,
    pub enabled: bool,
    pub active: bool,
    pub cpu_ctlr: u32,
    pub cpu_pmr: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct CpuInterfaceState {
    pub ctlr: u32,
    pub pmr: u32,
    pub bpr: u32,
}

/// Initialize the distributor + CPU interface for the boot core.
pub fn init_primary() -> Result<(), &'static str> {
    let state = ensure_state()?;

    unsafe {
        disable_distributor(state.dist_base);
        configure_timer_ppi_bank(state.dist_base);
        enable_distributor(state.dist_base);
        configure_cpu_interface(state.cpu_base);
        log_timer_ppi_config("primary", state.dist_base);
        log_cpu_interface_state_with_base("primary", state.cpu_base);
    }
    log_timer_irq_state("primary");

    DISTRIBUTOR_ENABLED.store(true, Ordering::Release);

    await_kernel_uart_println!(
        "[GIC] Distributor @ {:#x}, CPU interface @ {:#x} configured for boot core",
        state.dist_base,
        state.cpu_base
    );

    Ok(())
}

/// Configure the CPU interface + banked PPI registers for a secondary core.
pub fn init_secondary() -> Result<(), &'static str> {
    if !DISTRIBUTOR_ENABLED.load(Ordering::Acquire) {
        return Err("GIC distributor not initialized on the boot core");
    }

    let state = ensure_state()?;

    unsafe {
        configure_timer_ppi_bank(state.dist_base);
        configure_cpu_interface(state.cpu_base);
        log_timer_ppi_config("secondary", state.dist_base);
        log_cpu_interface_state_with_base("secondary", state.cpu_base);
    }
    log_timer_irq_state("secondary");

    await_kernel_uart_println!("[GIC] CPU interface armed on secondary core");

    Ok(())
}

pub fn cpu_interface_base() -> Option<usize> {
    ensure_state().ok().map(|state| state.cpu_base)
}

/// Snapshot of the timer PPI routing state for debugging.
pub fn timer_irq_snapshot() -> Option<TimerIrqState> {
    let state = ensure_state().ok()?;
    let idx = register_index(TIMER_PPI_ID);
    let bit = 1u32 << (TIMER_PPI_ID % 32);
    unsafe {
        let pending = (read32(state.dist_base + GICD_ISPENDR + idx * 4) & bit) != 0;
        let enabled = (read32(state.dist_base + GICD_ISENABLER + idx * 4) & bit) != 0;
        let active = (read32(state.dist_base + GICD_ISACTIVER + idx * 4) & bit) != 0;
        let cpu_ctlr = read32(state.cpu_base + GICC_CTLR);
        let cpu_pmr = read32(state.cpu_base + GICC_PMR);

        Some(TimerIrqState {
            pending,
            enabled,
            active,
            cpu_ctlr,
            cpu_pmr,
        })
    }
}

pub fn log_timer_irq_state(label: &str) {
    if let Some(snapshot) = timer_irq_snapshot() {
        await_kernel_uart_println!(
            "[GIC][timer][{}] pending={} enabled={} active={} | GICC_CTLR=0x{:02x} PMR=0x{:02x}",
            label,
            snapshot.pending,
            snapshot.enabled,
            snapshot.active,
            snapshot.cpu_ctlr,
            snapshot.cpu_pmr
        );
    } else {
        await_kernel_uart_println!("[GIC][timer][{}] snapshot unavailable", label);
    }
}

pub fn cpu_interface_state() -> Option<CpuInterfaceState> {
    let state = ensure_state().ok()?;
    unsafe {
        Some(CpuInterfaceState {
            ctlr: read32(state.cpu_base + GICC_CTLR),
            pmr: read32(state.cpu_base + GICC_PMR),
            bpr: read32(state.cpu_base + GICC_BPR),
        })
    }
}

pub fn log_cpu_interface_state(label: &str) {
    if let Some(snapshot) = cpu_interface_state() {
        await_kernel_uart_println!(
            "[GIC][cpu][{}] CTLR=0x{:02x} PMR=0x{:02x} BPR=0x{:02x}",
            label,
            snapshot.ctlr,
            snapshot.pmr,
            snapshot.bpr
        );
    } else {
        await_kernel_uart_println!("[GIC][cpu][{}] CPU interface snapshot unavailable", label);
    }
}

/// Force the timer PPI pending bit high for diagnostics.
pub fn force_timer_irq() {
    cpu::ensure_el1();

    let state = match ensure_state() {
        Ok(state) => state,
        Err(_) => return,
    };

    // Re-arm the local CNTP counter so the timer expires immediately even if the
    // distributor drop is ignored. This guarantees the hardware generates a
    // fresh edge, instead of relying solely on the GIC pending bit.
    unsafe {
        let now = CNTPCT_EL0.get();
        CNTP_CVAL_EL0.set(now.wrapping_add(1));
        CNTP_TVAL_EL0.set(1);
        let mut ctl = CNTP_CTL_EL0.get() | 0b1; // enable timer
        ctl &= !0b10; // unmask IRQ line
        CNTP_CTL_EL0.set(ctl);
        isb(SY);
        dsb(SY);
    }

    let idx = register_index(TIMER_PPI_ID);
    let bit = 1u32 << (TIMER_PPI_ID % 32);
    unsafe {
        write32(state.dist_base + GICD_ISPENDR + idx * 4, bit);
        dsb(SY);
    }
    await_kernel_uart_println!("[GIC][timer] Forced timer PPI pending via ISPENDR");
    log_timer_irq_state("force");
    unsafe {
        let iar = read32(state.cpu_base + GICC_IAR);
        let intid = iar & 0x3ff;
        match intid {
            id if id == TIMER_PPI_ID => {
                await_kernel_uart_println!(
                    "[GIC][timer][force] CPU interface latched timer INTID {} (raw=0x{:08x})",
                    id,
                    iar
                );
                write32(state.cpu_base + GICC_EOIR, iar);
                dsb(SY);
                await_kernel_uart_println!(
                    "[GIC][timer][force] Sent EOIR for forced timer INTID {}",
                    id
                );
            }
            SPURIOUS_IRQ_ID => {
                await_kernel_uart_println!(
                    "[GIC][timer][force] CPU interface returned spurious INTID (raw=0x{:08x})",
                    iar
                );
            }
            other => {
                await_kernel_uart_println!(
                    "[GIC][timer][force] CPU interface returned unexpected INTID {} (raw=0x{:08x})",
                    other,
                    iar
                );
            }
        }
    }
}

fn ensure_state() -> Result<GicState, &'static str> {
    unsafe {
        if let Some(state) = GIC_STATE {
            return Ok(state);
        }
    }

    let layout = dtb::peripherals_layout().ok_or("DTB summary missing while configuring GIC")?;
    let dist_base = layout
        .gic_distributor
        .try_into()
        .map_err(|_| "GIC distributor base does not fit usize")?;
    let cpu_base = layout
        .gic_redistributor
        .try_into()
        .map_err(|_| "GIC CPU interface base does not fit usize")?;

    if dist_base == 0 || cpu_base == 0 {
        return Err("DTB missing GIC-400 MMIO addresses");
    }

    let state = GicState {
        dist_base,
        cpu_base,
    };

    unsafe {
        GIC_STATE = Some(state);
    }

    Ok(state)
}

unsafe fn disable_distributor(dist_base: usize) {
    write32(dist_base + GICD_CTLR, 0);
    dsb(SY);
}

unsafe fn enable_distributor(dist_base: usize) {
    const CTLR_ENABLE_GRP0: u32 = 1 << 0;
    const CTLR_ENABLE_GRP1: u32 = 1 << 1;

    // Enable both Group0 (FIQ) and Group1 (IRQ) signalling so the NS world can receive IRQs.
    write32(dist_base + GICD_CTLR, CTLR_ENABLE_GRP0 | CTLR_ENABLE_GRP1);
    dsb(SY);
}

unsafe fn configure_timer_ppi_bank(dist_base: usize) {
    clear_pending(dist_base, TIMER_PPI_ID);
    set_group1(dist_base, TIMER_PPI_ID);
    set_priority(dist_base, TIMER_PPI_ID, 0x80);
    set_level_triggered(dist_base, TIMER_PPI_ID);
    enable_interrupt(dist_base, TIMER_PPI_ID);
    let core = (MPIDR_EL1.get() & 0xff) as u8;
    await_kernel_uart_println!(
        "[GIC][ppi][core {}] Timer PPI bank configured (group1, level, enabled)",
        core
    );
    dsb(SY);
}

unsafe fn configure_cpu_interface(cpu_base: usize) {
    // Disable before reconfiguration.
    write32(cpu_base + GICC_CTLR, 0);

    // Accept all IRQ priorities.
    write32(cpu_base + GICC_PMR, 0xFF);
    write32(cpu_base + GICC_BPR, 0);

    const CTLR_ENABLE_GRP0: u32 = 1 << 0;
    const CTLR_ENABLE_GRP1: u32 = 1 << 1;
    const CTLR_FIQ_BYPASS_DISABLE: u32 = 1 << 5;
    const CTLR_IRQ_BYPASS_DISABLE: u32 = 1 << 6;

    // Enable Group0/1 delivery and ensure IRQ/FIQ cannot bypass the GIC interface.
    write32(
        cpu_base + GICC_CTLR,
        CTLR_ENABLE_GRP0 | CTLR_ENABLE_GRP1 | CTLR_FIQ_BYPASS_DISABLE | CTLR_IRQ_BYPASS_DISABLE,
    );
    isb(SY);
}

unsafe fn log_cpu_interface_state_with_base(label: &str, cpu_base: usize) {
    let ctlr = read32(cpu_base + GICC_CTLR);
    let pmr = read32(cpu_base + GICC_PMR);
    let bpr = read32(cpu_base + GICC_BPR);
    await_kernel_uart_println!(
        "[GIC][cpu][{}] CTLR=0x{:02x} PMR=0x{:02x} BPR=0x{:02x}",
        label,
        ctlr,
        pmr,
        bpr
    );
}

unsafe fn log_timer_ppi_config(label: &str, dist_base: usize) {
    let idx = register_index(TIMER_PPI_ID);
    let bit = 1u32 << (TIMER_PPI_ID % 32);
    let group = read32(dist_base + GICD_IGROUPR + idx * 4);
    let enables = read32(dist_base + GICD_ISENABLER + idx * 4);
    let pend = read32(dist_base + GICD_ISPENDR + idx * 4);
    let active = read32(dist_base + GICD_ISACTIVER + idx * 4);
    let cfg_reg = dist_base + GICD_ICFGR + ((TIMER_PPI_ID as usize / 16) * 4);
    let cfg_shift = ((TIMER_PPI_ID % 16) * 2) as usize;
    let cfg_bits = (read32(cfg_reg) >> cfg_shift) & 0b11;
    let core = (MPIDR_EL1.get() & 0xff) as u8;

    await_kernel_uart_println!(
        "[GIC][ppi][{}][core {}] group1={} enabled={} pending={} active={} cfg=0b{:02b}",
        label,
        core,
        (group & bit) != 0,
        (enables & bit) != 0,
        (pend & bit) != 0,
        (active & bit) != 0,
        cfg_bits
    );
}

unsafe fn set_group1(dist_base: usize, int_id: u32) {
    let reg = dist_base + GICD_IGROUPR + register_index(int_id) * 4;
    let bit = 1u32 << (int_id % 32);
    let value = read32(reg) | bit;
    write32(reg, value);
}

unsafe fn set_priority(dist_base: usize, int_id: u32, priority: u8) {
    let addr = dist_base + GICD_IPRIORITYR + int_id as usize;
    write8(addr, priority);
}

unsafe fn set_level_triggered(dist_base: usize, int_id: u32) {
    let reg = dist_base + GICD_ICFGR + ((int_id as usize / 16) * 4);
    let shift = ((int_id % 16) * 2) as usize;
    let mut value = read32(reg);
    value &= !(0b11 << shift);
    value |= 0b10 << shift;
    write32(reg, value);
}

unsafe fn enable_interrupt(dist_base: usize, int_id: u32) {
    let reg = dist_base + GICD_ISENABLER + register_index(int_id) * 4;
    let bit = 1u32 << (int_id % 32);
    let value = read32(reg) | bit;
    write32(reg, value);
}

unsafe fn clear_pending(dist_base: usize, int_id: u32) {
    let reg = dist_base + GICD_ICPENDR + register_index(int_id) * 4;
    let bit = 1u32 << (int_id % 32);
    write32(reg, bit);
    dsb(SY);
}

const fn register_index(int_id: u32) -> usize {
    (int_id as usize) / 32
}

#[inline(always)]
unsafe fn read32(addr: usize) -> u32 {
    read_volatile(addr as *const u32)
}

#[inline(always)]
unsafe fn write32(addr: usize, value: u32) {
    write_volatile(addr as *mut u32, value);
}

#[inline(always)]
unsafe fn write8(addr: usize, value: u8) {
    write_volatile(addr as *mut u8, value);
}
