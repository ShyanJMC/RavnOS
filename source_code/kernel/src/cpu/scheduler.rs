// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Cooperative scheduler scaffolding for multi-core bring-up experiments.

use crate::await_kernel_uart_println;
use crate::bsp;
use crate::cpu::process::{self, ContextFrame, ProcessControlBlock};
use crate::cpu::{self, kernel_threads, userdebug_threads};
use crate::memory;
use aarch64_cpu::asm as cpu_asm;
use aarch64_cpu::asm::barrier::{isb, SY};
use aarch64_cpu::registers::{
    CNTFRQ_EL0, CNTKCTL_EL1, CNTP_CTL_EL0, CNTP_CVAL_EL0, CNTP_TVAL_EL0, CNTPCT_EL0, DAIF,
    TPIDR_EL0, TTBR0_EL1,
};
use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};
use tock_registers::interfaces::{Readable, Writeable};

/// Scheduler/timer configuration constants.
pub const MAX_CORES: usize = 4;
pub const MAX_KERNEL_TASKS: usize = 2;
pub const MAX_USER_TASKS: usize = 3;

/// Kernel task table executed exclusively by Core 0 (kernel domain).
#[no_mangle]
pub static mut KERNEL_TASKS: [unsafe extern "C" fn(); MAX_KERNEL_TASKS] =
    [kernel_task0, kernel_task1];

/// User/driver task table scheduled on the secondary cores (Core 1..MAX_CORES-1).
#[no_mangle]
pub static mut USER_TASKS: [unsafe extern "C" fn(); MAX_USER_TASKS] =
    [user_task0, user_task1, user_task2];

const KERNEL_TASK_NAMES: [&str; MAX_KERNEL_TASKS] = ["kernel_task0", "kernel_task1"];
const USER_TASK_NAMES: [&str; MAX_USER_TASKS] = ["user_task0", "user_task1", "user_task2"];
const KERNEL_TASK_PRIORITIES: [u32; MAX_KERNEL_TASKS] = [0, 1];
const USER_TASK_PRIORITIES: [u32; MAX_USER_TASKS] = [10, 11, 12];

static USER_TASK_TTBR0: [AtomicU64; MAX_USER_TASKS] = [const { AtomicU64::new(0) }; MAX_USER_TASKS];

/// Sample kernel tasks used when Core 0 is scheduled.
#[no_mangle]
pub unsafe extern "C" fn kernel_task0() {
    // TODO: Replace with real kernel task body.
    loop {
        kernel_threads::run_debug_checks();
        enable_irq();
        log_pre_wfi("kernel_task0");
        cpu_asm::wfi();
    }
}

#[no_mangle]
pub unsafe extern "C" fn kernel_task1() {
    // TODO: Replace with real kernel task body.
    loop {
        kernel_threads::run_debug_checks();
        enable_irq();
        log_pre_wfi("kernel_task1");
        cpu_asm::wfi();
    }
}

/// Sample user/driver tasks executed on cores 1..N.
#[no_mangle]
pub unsafe extern "C" fn user_task0() {
    // TODO: Replace with real user/driver task body.
    loop {
        userdebug_threads::run_debug_checks();
        enable_irq();
        log_pre_wfi("user_task0");
        cpu_asm::wfi();
    }
}

#[no_mangle]
pub unsafe extern "C" fn user_task1() {
    // TODO: Replace with real user/driver task body.
    loop {
        userdebug_threads::run_debug_checks();
        enable_irq();
        log_pre_wfi("user_task1");
        cpu_asm::wfi();
    }
}

#[no_mangle]
pub unsafe extern "C" fn user_task2() {
    // TODO: Replace with real user/driver task body.
    loop {
        userdebug_threads::run_debug_checks();
        enable_irq();
        log_pre_wfi("user_task2");
        cpu_asm::wfi();
    }
}

const VECTOR_TABLE_WORDS: usize = 512;
const VECTOR_SLOT_STRIDE_WORDS: usize = 0x80 / core::mem::size_of::<u32>();
const KERNEL_STACK_SIZE: usize = 4096;
const USER_STACK_SIZE: usize = 4096;
const STACK_ALIGN: u64 = 16;
const CONTEXT_GPR_COUNT: usize = 31;
const CONTEXT_FRAME_SIZE: usize = (CONTEXT_GPR_COUNT + 3) * core::mem::size_of::<u64>();
const CONTEXT_SP_OFFSET: usize = CONTEXT_GPR_COUNT * core::mem::size_of::<u64>();
const CONTEXT_ELR_OFFSET: usize = CONTEXT_SP_OFFSET + core::mem::size_of::<u64>();
const CONTEXT_SPSR_OFFSET: usize = CONTEXT_ELR_OFFSET + core::mem::size_of::<u64>();
const CONTEXT_LR_OFFSET: usize = (CONTEXT_GPR_COUNT - 1) * core::mem::size_of::<u64>();
const CONTEXT_X20_OFFSET: usize = 20 * core::mem::size_of::<u64>();
const CONTEXT_X21_OFFSET: usize = 21 * core::mem::size_of::<u64>();
const SPSR_KERNEL: u64 = 0b0101;
const SPSR_USER: u64 = 0b0000;

static mut CORE_RUNNING_SLOT: [Option<usize>; MAX_CORES] = [None; MAX_CORES];
static IRQ_WARNED: AtomicU64 = AtomicU64::new(0);
#[no_mangle]
static mut IRQ_HEARTBEAT: u64 = 0;

static mut KERNEL_TASK_STACKS: [[u8; KERNEL_STACK_SIZE]; MAX_KERNEL_TASKS] =
    [[0; KERNEL_STACK_SIZE]; MAX_KERNEL_TASKS];
static mut USER_TASK_STACKS: [[u8; USER_STACK_SIZE]; MAX_USER_TASKS] =
    [[0; USER_STACK_SIZE]; MAX_USER_TASKS];
const GICC_IAR_OFFSET: usize = 0x0C;
const GICC_EOIR_OFFSET: usize = 0x10;
const GICC_DIR_OFFSET: usize = 0x1000;

#[repr(C, align(16))]
pub struct ExceptionFrame {
    pub gprs: [u64; CONTEXT_GPR_COUNT],
    pub sp: u64,
    pub elr: u64,
    pub spsr: u64,
}

const fn build_vector_table() -> [u32; VECTOR_TABLE_WORDS] {
    let mut table = [0u32; VECTOR_TABLE_WORDS];

    table[VECTOR_SLOT_STRIDE_WORDS * 0] = 0x1400_0000; // 0x000: Synchronous EL1t
    table[VECTOR_SLOT_STRIDE_WORDS * 1] = 0x1400_0000; // 0x080: IRQ EL1t
    table[VECTOR_SLOT_STRIDE_WORDS * 2] = 0x1400_0000; // 0x100: FIQ EL1t
    table[VECTOR_SLOT_STRIDE_WORDS * 3] = 0x1400_0000; // 0x180: SError EL1t
    table[VECTOR_SLOT_STRIDE_WORDS * 4] = 0x1400_0000; // 0x200: Synchronous EL1h
    table[VECTOR_SLOT_STRIDE_WORDS * 5] = 0x9400_0000; // 0x280: IRQ EL1h (patched to bl scheduler)
    table[VECTOR_SLOT_STRIDE_WORDS * 6] = 0x1400_0000; // 0x300: FIQ EL1h
    table[VECTOR_SLOT_STRIDE_WORDS * 7] = 0x1400_0000; // 0x380: SError EL1h

    table
}

/// ARM64 Exception Vector Table aligned to 2 KiB (required by VBAR_EL1).
/// Each 0x80-byte slot reserves 128 bytes; we only need to seed the head opcode because the rest
/// of the slot remains zeroed. The boot assembly (`cpu/boot.s`) patches the EL1h IRQ slot (0x280)
/// at runtime so it issues a proper `bl scheduler_irq_handler` instruction with the correct imm26
/// offset.
#[repr(C, align(2048))]
pub struct VectorTable(pub [u32; VECTOR_TABLE_WORDS]);

#[no_mangle]
pub static VECTOR_TABLE: VectorTable = VectorTable(build_vector_table());

#[inline(always)]
const fn align_down(value: u64) -> u64 {
    value & !(STACK_ALIGN - 1)
}

fn log_pre_wfi(task_name: &str) {
    let core = crate::cpu::local_core_id();
    let daif = DAIF.get();
    await_kernel_uart_println!(
        "[DEBUG][task][pre-wfi] core {} {} DAIF=0x{:04x}",
        core,
        task_name,
        daif
    );
    if (daif & (1 << 7)) != 0 {
        await_kernel_uart_println!(
            "[DEBUG][task][pre-wfi] {} WARNING: DAIF.I is set before WFI",
            task_name
        );
    }
}

unsafe fn kernel_stack_region(slot: usize) -> (u64, u64) {
    let base = KERNEL_TASK_STACKS.get_unchecked(slot).as_ptr() as u64;
    let top = align_down(base + KERNEL_STACK_SIZE as u64);
    await_kernel_uart_println!("[DEBUG] Kernel stack region:\n\tbase:{base}\n\ttop:{top}");
    (base, top)
}

unsafe fn user_stack_region(slot: usize) -> (u64, u64) {
    let base = USER_TASK_STACKS.get_unchecked(slot).as_ptr() as u64;
    let top = align_down(base + USER_STACK_SIZE as u64);
    await_kernel_uart_println!("[DEBUG] User stack region:\n\tbase:{base}\n\ttop:{top}");
    (base, top)
}

/// Publish VECTOR_TABLE so the CPU jumps to our handlers on every exception/IRQ.
/// Must be invoked from the early kernel init code before enabling interrupts.
pub unsafe fn install_vector_table() {
    let addr = &VECTOR_TABLE as *const _ as usize;
    asm!(
        "msr vbar_el1, {0}",
        in(reg) addr,
        options(nostack, preserves_flags)
    );
    // Ensure this core's I-cache observes the patched vector slot.
    unsafe {
        asm!(
            "dsb sy",
            "isb",
            "ic iallu",
            "dsb sy",
            "isb",
            options(nostack, preserves_flags)
        );
    }
    let core = crate::cpu::local_core_id();
    let vbar: u64;
    unsafe {
        asm!("mrs {0}, vbar_el1", out(reg) vbar, options(nomem, preserves_flags));
    }
    await_kernel_uart_println!(
        "[DEBUG][vector] install_vector_table core {} VBAR_EL1=0x{:016x}",
        core,
        vbar
    );
}

/// Clears the IRQ mask bit in DAIF so timer interrupts are visible at EL1.
pub unsafe fn enable_irq() {
    let daif_before = DAIF.get();
    await_kernel_uart_println!(
        "[DEBUG][irq] enable_irq entry DAIF=0x{:04x}",
        daif_before
    );
    if let Some(cpu) = bsp::cpu_interface_state() {
        await_kernel_uart_println!(
            "[DEBUG][irq] enable_irq entry GICC_CTLR=0x{:02x} PMR=0x{:02x} BPR=0x{:02x}",
            cpu.ctlr,
            cpu.pmr,
            cpu.bpr
        );
    } else {
        await_kernel_uart_println!("[DEBUG][irq] enable_irq entry missing CPU interface state");
    }
    // Clear both IRQ (I) and FIQ (F) masks to be safe across Group0/1 routing.
    asm!("msr daifclr, #3", options(nostack, preserves_flags));
    let daif = DAIF.get();
    if (daif & (1 << 7)) != 0 && IRQ_WARNED.load(Ordering::Relaxed) == 0 {
        IRQ_WARNED.store(1, Ordering::Relaxed);
        await_kernel_uart_println!(
            "[DEBUG][irq] DAIF.I still set after enable_irq() (daif=0x{:04x}); check call site",
            daif
        );
    }
    let cnt_ctl = CNTP_CTL_EL0.get();
    let cnt_tval = CNTP_TVAL_EL0.get();
    let cnt_cval = CNTP_CVAL_EL0.get();
    let vbar: u64;
    unsafe {
        asm!("mrs {0}, vbar_el1", out(reg) vbar, options(nomem, preserves_flags));
    }
    await_kernel_uart_println!(
        "[DEBUG][irq] enable_irq exit DAIF=0x{:04x} CNTP_CTL=0x{:02x} TVAL={} CVAL={}",
        daif,
        cnt_ctl,
        cnt_tval,
        cnt_cval
    );
    await_kernel_uart_println!(
        "[DEBUG][irq] enable_irq exit VBAR_EL1=0x{:016x}",
        vbar
    );
    bsp::log_cpu_interface_state("[irq] enable_irq");
    snapshot_timer_irq_path("[irq] enable_irq", None, None);
}

pub fn log_irq_vector_slot(label: &str) {
    log_vector_slot(label, "EL1t", 0x080);
    log_vector_slot(label, "EL1h", 0x280);
}

fn log_vector_slot(label: &str, mode: &str, offset_bytes: usize) {
    let base = &VECTOR_TABLE as *const _ as usize;
    let slot_addr = base + offset_bytes;
    let entry = unsafe { core::ptr::read_volatile(slot_addr as *const u32) };
    let imm26 = (entry & 0x03ff_ffff) as i64;
    let offset = ((imm26 << 2) << 36) >> 36; // sign-extend 28-bit offset
    let target = slot_addr as i64 + 4 + offset;
    let handler = scheduler_irq_handler as *const () as usize;
    await_kernel_uart_println!(
        "[DEBUG][vector] {} {} IRQ slot entry=0x{:08x} target=0x{:016x} handler=0x{:016x}",
        label,
        mode,
        entry,
        target as usize,
        handler
    );
}

pub fn read_irq_heartbeat() -> u64 {
    unsafe { core::ptr::read_volatile(&IRQ_HEARTBEAT) }
}

/// Programs the per-core Generic Timer so it fires an IRQ roughly every 5 ms.
/// Sequence (per core):
/// - Read `cntfrq_el0` to learn the fixed timer frequency.
/// - Compute ticks = freq * 0.005 seconds.
/// - Write the countdown value into `cntp_tval_el0`.
/// - Enable the timer/IRQ through `cntp_ctl_el0`.
pub unsafe fn setup_generic_timer_5ms() {
    cpu::ensure_el1();
    let freq = CNTFRQ_EL0.get();
    let ticks = freq / 200;
    let now = CNTPCT_EL0.get();

    CNTP_CVAL_EL0.set(now.wrapping_add(ticks));
    CNTP_TVAL_EL0.set(ticks);
    let mut ctl = CNTP_CTL_EL0.get();
    ctl |= 0b1; // enable timer
    ctl &= !0b10; // unmask IRQ
    CNTP_CTL_EL0.set(ctl);
    isb(SY);

    snapshot_timer_irq_path("[timer] setup_generic_timer_5ms", Some(freq), Some(ticks));
    bsp::log_cpu_interface_state("[timer] setup_generic_timer_5ms");
}

fn snapshot_timer_irq_path(label: &str, freq: Option<u64>, ticks: Option<u64>) {
    if let (Some(freq), Some(ticks)) = (freq, ticks) {
        let approx_us = if freq != 0 {
            (ticks.saturating_mul(1_000_000)) / freq
        } else {
            0
        };
        await_kernel_uart_println!(
            "[DEBUG][timer] {} freq={} Hz ticks={} (~{} us)",
            label,
            freq,
            ticks,
            approx_us
        );
    } else {
        await_kernel_uart_println!("[DEBUG][timer] {} (freq/ticks unavailable)", label);
    }

    let cntp_ctl = CNTP_CTL_EL0.get();
    let cntp_tval = CNTP_TVAL_EL0.get();
    let cntp_cval = CNTP_CVAL_EL0.get();
    let cntkctl = CNTKCTL_EL1.get();
    let daif = DAIF.get();
    let enable = (cntp_ctl & 0b1) != 0;
    let mask = (cntp_ctl & 0b10) != 0;
    let pending = (cntp_ctl & 0b100) != 0;

    await_kernel_uart_println!(
        "[DEBUG][timer] {} CNTP_CTL={{enable:{} mask:{} pending:{}}} TVAL={} CVAL={} CNTKCTL=0x{:04x} DAIF=0x{:04x}",
        label,
        enable,
        mask,
        pending,
        cntp_tval,
        cntp_cval,
        cntkctl,
        daif
    );

    if let Some(snapshot) = bsp::timer_irq_snapshot() {
        await_kernel_uart_println!(
            "[DEBUG][timer] {} GIC pending={} enabled={} active={} | GICC_CTLR=0x{:02x} PMR=0x{:02x}",
            label,
            snapshot.pending,
            snapshot.enabled,
            snapshot.active,
            snapshot.cpu_ctlr,
            snapshot.cpu_pmr
        );
    } else {
        await_kernel_uart_println!(
            "[DEBUG][timer] {} GIC timer snapshot unavailable (interrupt controller not ready)",
            label
        );
    }
}

/// Emit a consolidated scheduler snapshot so other modules can request verbose traces on demand.
pub fn log_scheduler_snapshot(label: &str, core_id: usize) {
    if core_id >= MAX_CORES {
        await_kernel_uart_println!(
            "[DEBUG][sched][snapshot] {} invalid core {} (max {})",
            label,
            core_id,
            MAX_CORES - 1
        );
        return;
    }

    let domain = if core_id == 0 { "kernel" } else { "user" };
    let slot = unsafe { CORE_RUNNING_SLOT[core_id] };
    let ttbr0 = TTBR0_EL1.get();
    await_kernel_uart_println!(
        "[DEBUG][sched][snapshot] {} core {} domain {} slot {:?} TTBR0_EL1=0x{:016x}",
        label,
        core_id,
        domain,
        slot,
        ttbr0
    );

    snapshot_timer_irq_path("[sched][snapshot]", None, None);
    log_irq_vector_slot("[sched][snapshot]");

    match (domain, slot) {
        ("kernel", Some(idx)) => {
            let _ = process::with_kernel_process(idx, |pcb| {
                await_kernel_uart_println!(
                    "[DEBUG][sched][snapshot] kernel slot {} pid {} state {:?} priority {} sp {:#018x} pc {:#018x}",
                    idx,
                    pcb.pid,
                    pcb.state,
                    pcb.priority,
                    pcb.sp,
                    pcb.pc
                );
            });
        }
        ("user", Some(idx)) => {
            let _ = process::with_user_process(idx, |pcb| {
                await_kernel_uart_println!(
                    "[DEBUG][sched][snapshot] user slot {} pid {} state {:?} priority {} ttbr0 {:#018x}",
                    idx,
                    pcb.pid,
                    pcb.state,
                    pcb.priority,
                    pcb.ttbr0
                );
            });
        }
        _ => {
            await_kernel_uart_println!(
                "[DEBUG][sched][snapshot] {} core {} no active PCB slot",
                label,
                core_id
            );
        }
    }
}

/// Timer IRQ handler that implements a preemptive multi-core scheduler.
#[no_mangle]
pub unsafe extern "C" fn scheduler_irq_handler() {
    let cpu_if_base =
        bsp::interrupt_controller_cpu_base().expect("GIC CPU interface not initialized");
    asm!(
        "sub sp, sp, {frame_size}",
        "stp x0, x1, [sp, #0]",
        "stp x2, x3, [sp, #16]",
        "stp x4, x5, [sp, #32]",
        "stp x6, x7, [sp, #48]",
        "stp x8, x9, [sp, #64]",
        "stp x10, x11, [sp, #80]",
        "stp x12, x13, [sp, #96]",
        "stp x14, x15, [sp, #112]",
        "stp x16, x17, [sp, #128]",
        "stp x18, x19, [sp, #144]",
        "stp x20, x21, [sp, #160]",
        "stp x22, x23, [sp, #176]",
        "stp x24, x25, [sp, #192]",
        "stp x26, x27, [sp, #208]",
        "stp x28, x29, [sp, #224]",
        "str x30, [sp, #{lr_off}]",
        "mrs x11, elr_el1",
        "str x11, [sp, #{elr_off}]",
        "mrs x12, spsr_el1",
        "str x12, [sp, #{spsr_off}]",
        "and x13, x12, #0b11111",
        "cmp x13, #0b0000",
        "b.ne 1f",
        "mrs x10, sp_el0",
        "b 2f",
        "1:",
        "add x10, sp, {frame_size}",
        "2:",
        "str x10, [sp, #{sp_off}]",
        "mrs x9, cntpct_el0",
        "adrp x10, {heartbeat}",
        "add x10, x10, :lo12:{heartbeat}",
        "str x9, [x10]",

        "mov x0, #0",
        "bl {probe}",

        "ldr w9, [{cpu_if}, #{gicc_iar}]",
        "mrs x0, mpidr_el1",
        "and x0, x0, #0b11",
        "mov x28, x0",
        "mov x19, x9",
        "mov x0, #1",
        "bl {probe}",
        "mrs x2, daif",
        "mov x0, x28",
        "mov x1, x19",
        "bl {log_entry}",
        "mov x9, x19",
        "mov x0, x28",
        "mov x1, sp",
        "bl {tick}",

        "mov x20, x0",
        "mov x0, #2",
        "bl {probe}",
        "sub sp, sp, #32",
        "str x20, [sp, #0]",
        "str x13, [sp, #8]",
        "str w9, [sp, #24]",

        "mrs x10, cntfrq_el0",
        "mov x11, #200",
        "udiv x10, x10, x11",
        "msr cntp_tval_el0, x10",

        "ldr x12, [x20, #{sp_off}]",
        "str x12, [sp, #16]",
        "ldr x13, [x20, #{elr_off}]",
        "ldr x14, [x20, #{spsr_off}]",
        "msr elr_el1, x13",
        "msr spsr_el1, x14",

        "ldp x0, x1, [x20, #0]",
        "ldp x2, x3, [x20, #16]",
        "ldp x4, x5, [x20, #32]",
        "ldp x6, x7, [x20, #48]",
        "ldp x8, x9, [x20, #64]",
        "ldp x10, x11, [x20, #80]",
        "ldp x12, x13, [x20, #96]",
        "ldp x14, x15, [x20, #112]",
        "ldp x16, x17, [x20, #128]",
        "ldp x18, x19, [x20, #144]",
        "ldp x22, x23, [x20, #176]",
        "ldp x24, x25, [x20, #192]",
        "ldp x26, x27, [x20, #208]",
        "ldp x28, x29, [x20, #224]",
        "ldr x30, [x20, #{lr_off}]",

        "ldr x9, [sp, #0]",
        "ldr x21, [x9, #{x21_off}]",
        "ldr x20, [x9, #{x20_off}]",
        "ldr x15, [sp, #8]",
        "ldr x12, [sp, #16]",
        "ldr w10, [sp, #24]",
        "add sp, sp, #32",
        "add sp, sp, {frame_size}",
        "cmp x15, #0",
        "b.ne 3f",
        "msr sp_el0, x12",
        "b 4f",
        "3:",
        "mov sp, x12",
        "4:",

        "mov w9, w10",
        "str w9, [{cpu_if}, #{gicc_eoir}]",
        "str w9, [{cpu_if}, #{gicc_dir}]",
        "mov x0, #3",
        "bl {probe}",
        "dsb sy",
        "eret",
        cpu_if = in(reg) cpu_if_base,
        gicc_iar = const GICC_IAR_OFFSET,
        gicc_eoir = const GICC_EOIR_OFFSET,
        gicc_dir = const GICC_DIR_OFFSET,
        tick = sym scheduler_preempt_tick,
        log_entry = sym log_irq_handler_entry,
        probe = sym scheduler_irq_probe,
        frame_size = const CONTEXT_FRAME_SIZE,
        lr_off = const CONTEXT_LR_OFFSET,
        sp_off = const CONTEXT_SP_OFFSET,
        elr_off = const CONTEXT_ELR_OFFSET,
        spsr_off = const CONTEXT_SPSR_OFFSET,
        x20_off = const CONTEXT_X20_OFFSET,
        x21_off = const CONTEXT_X21_OFFSET,
        heartbeat = sym IRQ_HEARTBEAT,
        options(noreturn)
    );
}

fn log_irq_handler_entry(core_id: usize, iar: u32, daif: u64) {
    await_kernel_uart_println!(
        "[DEBUG][irq] handler enter core {} IAR=0x{:08x} DAIF=0x{:04x}",
        core_id,
        iar,
        daif
    );
    if (daif & (1 << 7)) != 0 {
        await_kernel_uart_println!(
            "[DEBUG][irq] handler enter core {} WARNING: DAIF.I set on entry",
            core_id
        );
    }
}

// === Scheduler bootstrap + call-by-call walkthrough ========================
// Minimal main.rs bring-up (runs once on each core during init):
// unsafe {
//     cpu::scheduler::install_vector_table();    // Publishes VECTOR_TABLE via VBAR_EL1; boot.s patches
//                                                // slot 0x280 so it branches to scheduler_irq_handler.
//     cpu::scheduler::setup_generic_timer_5ms(); // Arms the per-core timer to emit IRQs every 5 ms.
//     cpu::scheduler::enable_irq();              // Clears DAIF.I so IRQs are delivered.
// }
// After that point:
//  - main.rs can park in a loop or start higher-level services; the timer IRQ will keep firing.
//  - kernel_main() iterates over core IDs 1..N and calls bsp::start_secondary_core(core_id) so every
//    secondary CPU reuses the same init sequence: install vector table, arm its timer, unmask IRQs.
//
// IRQ/vector-table story:
// 1) The per-core Generic Timer counts down from cntp_tval_el0 to zero.
// 2) Once zero is reached, hardware raises an IRQ and indexes VBAR_EL1 + 0x280 (EL1h IRQ slot).
// 3) Boot.s rewrites slot 0x280 so the 32-bit opcode becomes `bl scheduler_irq_handler`, i.e. the
//    imm26 field is filled in with the correct offset toward the Rust handler.
// 4) Execution continues inside scheduler_irq_handler() (Rust symbol), still at EL1h.
//
// scheduler_irq_handler() flow (per instruction in the asm! block):
// 1) Spill every GPR plus ELR/SPSR into a temporary ExceptionFrame on the current stack.
// 2) Acknowledge the GIC interrupt and call scheduler_preempt_tick(core_id, frame_ptr).
// 3) The Rust helper copies the frame into the PCB belonging to the interrupted task, picks the next
//    runnable PCB for the domain (kernel on core 0, user tasks on cores 1..N), and returns a pointer
//    to the ContextFrame that should be restored.
// 4) The handler reprograms cntp_tval_el0 for the next 5 ms quantum, frees the temporary frame, loads
//    the saved SP/ELR/SPSR from the selected ContextFrame, and restores all GPRs.
// 5) It deactivates the interrupt (EOIR + DIR), switches SP to the target task, and executes `eret`,
//    resuming execution exactly where the chosen PCB left off.
//
// Cores that execute USER_TASKS:
// - Every secondary core (1..MAX_CORES-1) runs exclusively out of the USER_TASKS PCB table. When a
//   slot publishes a TTBR0 value, the scheduler programs TTBR0_EL1 before resuming that task so each
//   context uses its own address space.

#[no_mangle]
pub unsafe extern "C" fn scheduler_preempt_tick(
    core_id: usize,
    frame_ptr: *const ExceptionFrame,
) -> *const ContextFrame {
    let frame = &*frame_ptr;
    let domain = if core_id == 0 { "kernel" } else { "user" };
    let prev_slot_idx = unsafe {
        CORE_RUNNING_SLOT[core_id]
            .map(|slot| slot as isize)
            .unwrap_or(-1)
    };
    await_kernel_uart_println!(
        "[DEBUG][sched][1][tick-entry] core {} ({}) prev_slot {} elr {:#018x} sp {:#018x} spsr {:#010x}",
        core_id,
        domain,
        prev_slot_idx,
        frame.elr,
        frame.sp,
        frame.spsr
    );

    let next_ctx = if core_id == 0 {
        handle_kernel_tick(core_id, frame)
    } else {
        handle_user_tick(core_id, frame)
    };

    let next_slot_idx = unsafe {
        CORE_RUNNING_SLOT[core_id]
            .map(|slot| slot as isize)
            .unwrap_or(-1)
    };
    await_kernel_uart_println!(
        "[DEBUG][sched][2][tick-exit] core {} ({}) next_slot {} ctx_ptr {:#018x}",
        core_id,
        domain,
        next_slot_idx,
        next_ctx as u64
    );

    next_ctx
}

fn handle_kernel_tick(core_id: usize, frame: &ExceptionFrame) -> *const ContextFrame {
    if let Some(slot) = unsafe { CORE_RUNNING_SLOT[core_id] } {
        await_kernel_uart_println!(
            "[DEBUG][sched][3][kernel-save] core {} slot {} elr {:#018x} sp {:#018x}",
            core_id,
            slot,
            frame.elr,
            frame.sp
        );
        save_kernel_context(slot, frame);
        mark_kernel_slot_idle(slot);
    }
    let next_slot = advance_slot(core_id, MAX_KERNEL_TASKS);
    unsafe {
        CORE_RUNNING_SLOT[core_id] = Some(next_slot);
    }
    mark_kernel_slot_running(next_slot, core_id);
    let ctx = load_kernel_context(next_slot);
    await_kernel_uart_println!(
        "[DEBUG][sched][4][kernel-next] core {} slot {} ctx_ptr {:#018x}",
        core_id,
        next_slot,
        ctx as u64
    );
    ctx
}

fn handle_user_tick(core_id: usize, frame: &ExceptionFrame) -> *const ContextFrame {
    if let Some(slot) = unsafe { CORE_RUNNING_SLOT[core_id] } {
        await_kernel_uart_println!(
            "[DEBUG][sched][5][user-save] core {} slot {} elr {:#018x} sp {:#018x}",
            core_id,
            slot,
            frame.elr,
            frame.sp
        );
        save_user_context(slot, frame);
        mark_user_slot_idle(slot);
    }
    let next_slot = advance_slot(core_id, MAX_USER_TASKS);
    unsafe {
        CORE_RUNNING_SLOT[core_id] = Some(next_slot);
    }
    mark_user_slot_running(next_slot, core_id);
    configure_user_ttbr(core_id, next_slot);
    let ctx = load_user_context(next_slot);
    await_kernel_uart_println!(
        "[DEBUG][sched][6][user-next] core {} slot {} ctx_ptr {:#018x}",
        core_id,
        next_slot,
        ctx as u64
    );
    ctx
}

fn advance_slot(core_id: usize, span: usize) -> usize {
    if span == 0 {
        return 0;
    }
    let current = unsafe { CORE_RUNNING_SLOT[core_id] };
    let next = match current {
        Some(current) => (current + 1) % span,
        None => 0,
    };
    await_kernel_uart_println!(
        "[DEBUG][sched][7][advance] core {} span {} current {:?} -> next {}",
        core_id,
        span,
        current,
        next
    );
    next
}

fn save_kernel_context(slot: usize, frame: &ExceptionFrame) {
    await_kernel_uart_println!(
        "[DEBUG][sched][8][kernel-copy] slot {} sp {:#018x} elr {:#018x}",
        slot,
        frame.sp,
        frame.elr
    );
    let _ = process::with_kernel_process_mut(slot, |pcb| {
        update_pcb_from_frame(pcb, frame);
    });
}

fn save_user_context(slot: usize, frame: &ExceptionFrame) {
    await_kernel_uart_println!(
        "[DEBUG][sched][9][user-copy] slot {} sp {:#018x} elr {:#018x}",
        slot,
        frame.sp,
        frame.elr
    );
    let _ = process::with_user_process_mut(slot, |pcb| {
        update_pcb_from_frame(pcb, frame);
    });
}

fn update_pcb_from_frame(pcb: &mut ProcessControlBlock, frame: &ExceptionFrame) {
    pcb.context
        .copy_from_parts(&frame.gprs, frame.sp, frame.elr, frame.spsr);
    pcb.sp = frame.sp;
    pcb.lr = frame.gprs[30];
    pcb.pc = frame.elr;
    pcb.spsr_el1 = frame.spsr;
    pcb.registers.copy_from_slice(&frame.gprs[..30]);
}

fn load_kernel_context(slot: usize) -> *const ContextFrame {
    let ctx = process::with_kernel_process(slot, |pcb| pcb.context.as_ptr())
        .unwrap_or_else(|| panic!("[sched] Kernel slot {slot} missing during context switch"));
    await_kernel_uart_println!(
        "[DEBUG][sched][10][kernel-load] slot {} ctx_ptr {:#018x}",
        slot,
        ctx as u64
    );
    ctx
}

fn load_user_context(slot: usize) -> *const ContextFrame {
    let ctx = process::with_user_process(slot, |pcb| pcb.context.as_ptr())
        .unwrap_or_else(|| panic!("[sched] User slot {slot} missing during context switch"));
    await_kernel_uart_println!(
        "[DEBUG][sched][11][user-load] slot {} ctx_ptr {:#018x}",
        slot,
        ctx as u64
    );
    ctx
}

fn configure_user_ttbr(core_id: usize, slot: usize) {
    let _ = process::with_user_process(slot, |pcb| {
        if pcb.ttbr0 != 0 {
            await_kernel_uart_println!(
                "[DEBUG][sched][12][ttbr] core {} slot {} ttbr0 {:#018x} tpidr {:#018x}",
                core_id,
                slot,
                pcb.ttbr0,
                pcb.tpidr_el0
            );
            TTBR0_EL1.set(pcb.ttbr0);
            let identity = ((core_id as u64) << 32) | (pcb.tpidr_el0 & 0xffff_ffff);
            TPIDR_EL0.set(identity);
            isb(SY);
        } else {
            await_kernel_uart_println!(
                "[DEBUG][sched][12][ttbr-skip] core {} slot {} missing ttbr0",
                core_id,
                slot
            );
        }
    });
}

fn mark_kernel_slot_idle(slot: usize) {
    let _ = process::with_kernel_process_mut(slot, |pcb| {
        pcb.mark_idle();
    });
}

fn mark_kernel_slot_running(slot: usize, core_id: usize) {
    let _ = process::with_kernel_process_mut(slot, |pcb| {
        pcb.mark_running(core_id as u8);
    });
}

fn mark_user_slot_idle(slot: usize) {
    let _ = process::with_user_process_mut(slot, |pcb| {
        pcb.mark_idle();
    });
}

fn mark_user_slot_running(slot: usize, core_id: usize) {
    let _ = process::with_user_process_mut(slot, |pcb| {
        pcb.mark_running(core_id as u8);
    });
}

/// Returns the slot index currently assigned to `core_id`.
pub fn current_task_slot(core_id: usize) -> usize {
    if core_id >= MAX_CORES {
        return 0;
    }
    unsafe { CORE_RUNNING_SLOT[core_id].unwrap_or(0) }
}

/// Stores the TTBR0 physical address backing a cooperative user task slot.
pub fn set_user_task_ttbr(slot: usize, ttbr0_phys: u64) {
    if slot >= MAX_USER_TASKS {
        await_kernel_uart_println!(
            "[sched] Ignoring TTBR0 assignment for invalid user task slot {}",
            slot
        );
        return;
    }
    USER_TASK_TTBR0[slot].store(ttbr0_phys, Ordering::Release);
}

/// Initializes the kernel PCB table with the cooperative debug tasks.
pub fn init_kernel_process_descriptors() {
    let kernel_ttbr1 = memory::kernel_ttbr1_phys().unwrap_or(0);
    process::with_kernel_process_table(|table| {
        for (slot, task) in unsafe { KERNEL_TASKS.iter().enumerate() } {
            let priority = *KERNEL_TASK_PRIORITIES.get(slot).unwrap_or(&0);
            let name = *KERNEL_TASK_NAMES.get(slot).unwrap_or(&"kernel_task");
            let (stack_base, stack_top) = unsafe { kernel_stack_region(slot) };
            table.register_kernel_task(
                slot,
                *task,
                priority,
                kernel_ttbr1,
                name,
                stack_base,
                stack_top,
                SPSR_KERNEL,
            );
        }
    });
}

/// Initializes the user PCB table with the cooperative debug tasks.
pub fn init_user_process_descriptors() {
    process::with_user_process_table(|table| {
        for (slot, task) in unsafe { USER_TASKS.iter().enumerate() } {
            let ttbr0 = USER_TASK_TTBR0[slot].load(Ordering::Acquire);
            if ttbr0 == 0 {
                await_kernel_uart_println!(
                    "[sched] WARNING: TTBR0 for user task slot {} not initialized; skipping PCB registration",
                    slot
                );
                continue;
            }
            let priority = *USER_TASK_PRIORITIES.get(slot).unwrap_or(&0);
            let name = *USER_TASK_NAMES.get(slot).unwrap_or(&"user_task");
            let (stack_base, stack_top) = unsafe { user_stack_region(slot) };
            table.register_user_task(
                slot, *task, priority, ttbr0, name, stack_base, stack_top, SPSR_USER,
            );
        }
    });
}
#[no_mangle]
pub unsafe extern "C" fn scheduler_irq_probe(marker: u64) {
    await_kernel_uart_println!("[DEBUG][irq] probe marker={}", marker);
}
