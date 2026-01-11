// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Cooperative scheduler scaffolding for multi-core bring-up experiments.

use crate::cpu::{kernel_threads, userdebug_threads};
use core::arch::asm;

/// Scheduler/timer configuration constants.
pub const MAX_CORES: usize = 4;
pub const MAX_KERNEL_TASKS: usize = 2;
pub const MAX_USER_TASKS: usize = 3;

/// Per-core index of the task that should run next.
#[no_mangle]
pub static mut CURRENT_TASK_IDX: [usize; MAX_CORES] = [0; MAX_CORES];

/// Kernel task table executed exclusively by Core 0 (kernel domain).
#[no_mangle]
pub static mut KERNEL_TASKS: [unsafe extern "C" fn(); MAX_KERNEL_TASKS] =
    [kernel_task0, kernel_task1];

/// User/driver task table scheduled on the secondary cores (Core 1..MAX_CORES-1).
#[no_mangle]
pub static mut USER_TASKS: [unsafe extern "C" fn(); MAX_USER_TASKS] =
    [user_task0, user_task1, user_task2];

/// Sample kernel tasks used when Core 0 is scheduled.
#[no_mangle]
pub unsafe extern "C" fn kernel_task0() {
    // TODO: Replace with real kernel task body.
    kernel_threads::run_debug_checks();
}

#[no_mangle]
pub unsafe extern "C" fn kernel_task1() {
    // TODO: Replace with real kernel task body.
    kernel_threads::run_debug_checks();
}

/// Sample user/driver tasks executed on cores 1..N.
#[no_mangle]
pub unsafe extern "C" fn user_task0() {
    // TODO: Replace with real user/driver task body.
    userdebug_threads::run_debug_checks();
}

#[no_mangle]
pub unsafe extern "C" fn user_task1() {
    // TODO: Replace with real user/driver task body.
    userdebug_threads::run_debug_checks();
}

#[no_mangle]
pub unsafe extern "C" fn user_task2() {
    // TODO: Replace with real user/driver task body.
    userdebug_threads::run_debug_checks();
}

const VECTOR_TABLE_WORDS: usize = 512;
const VECTOR_SLOT_STRIDE_WORDS: usize = 0x20 / core::mem::size_of::<u32>();

const fn build_vector_table() -> [u32; VECTOR_TABLE_WORDS] {
    let mut table = [0u32; VECTOR_TABLE_WORDS];

    table[VECTOR_SLOT_STRIDE_WORDS * 0] = 0x1400_0000; // 0x000: Synchronous EL1t
    table[VECTOR_SLOT_STRIDE_WORDS * 1] = 0x1400_0000; // 0x020: IRQ EL1t
    table[VECTOR_SLOT_STRIDE_WORDS * 2] = 0x1400_0000; // 0x040: FIQ EL1t
    table[VECTOR_SLOT_STRIDE_WORDS * 3] = 0x1400_0000; // 0x060: SError EL1t
    table[VECTOR_SLOT_STRIDE_WORDS * 4] = 0x1400_0000; // 0x080: Synchronous EL1h
    table[VECTOR_SLOT_STRIDE_WORDS * 5] = 0x9400_0000; // 0x0A0: IRQ EL1h (patched to bl scheduler)
    table[VECTOR_SLOT_STRIDE_WORDS * 6] = 0x1400_0000; // 0x0C0: FIQ EL1h
    table[VECTOR_SLOT_STRIDE_WORDS * 7] = 0x1400_0000; // 0x0E0: SError EL1h

    table
}

/// ARM64 Exception Vector Table aligned to 2 KiB (required by VBAR_EL1).
/// Each 0x20-byte slot reserves 32 bytes; we only need to seed the head opcode because the rest
/// of the slot remains zeroed. The boot assembly (`cpu/boot.s`) patches the EL1h IRQ slot (0x0A0)
/// at runtime so it issues a proper `bl scheduler_irq_handler` instruction with the correct imm26
/// offset.
#[repr(C, align(2048))]
pub struct VectorTable(pub [u32; VECTOR_TABLE_WORDS]);

#[no_mangle]
pub static VECTOR_TABLE: VectorTable = VectorTable(build_vector_table());

/// Publish VECTOR_TABLE so the CPU jumps to our handlers on every exception/IRQ.
/// Must be invoked from the early kernel init code before enabling interrupts.
pub unsafe fn install_vector_table() {
    let addr = &VECTOR_TABLE as *const _ as usize;
    asm!(
        "msr vbar_el1, {0}",
        in(reg) addr,
        options(nostack, preserves_flags)
    );
}

/// Clears the IRQ mask bit in DAIF so timer interrupts are visible at EL1.
pub unsafe fn enable_irq() {
    asm!("msr daifclr, #2", options(nostack, preserves_flags));
}

/// Programs the per-core Generic Timer so it fires an IRQ roughly every 5 ms.
/// Sequence (per core):
/// - Read `cntfrq_el0` to learn the fixed timer frequency.
/// - Compute ticks = freq * 0.005 seconds.
/// - Write the countdown value into `cntp_tval_el0`.
/// - Enable the timer/IRQ through `cntp_ctl_el0`.
pub unsafe fn setup_generic_timer_5ms() {
    let mut freq: u64;
    let ticks: u64;

    // Read the timer frequency in Hz from cntfrq_el0.
    asm!(
        "mrs {freq:x}, cntfrq_el0",
        freq = out(reg) freq
    );
    // Convert frequency to ticks for 5 ms (ticks = freq / 200).
    ticks = freq / 200;

    // Program the countdown and enable the timer so it raises an IRQ when it reaches zero.
    asm!(
        "msr cntp_tval_el0, {ticks}", // Initial countdown value (per core)
        "mov x0, #1",
        "msr cntp_ctl_el0, x0",       // Enable timer + IRQ routing
        ticks = in(reg) ticks,
        out("x0") _
    );
}

/// Timer IRQ handler that implements a cooperative multi-core scheduler.
/// Core 0 rotates over KERNEL_TASKS; cores 1..N rotate over USER_TASKS.
#[no_mangle]
pub unsafe extern "C" fn scheduler_irq_handler() {
    asm!(
        // Re-arm the timer with the same 5 ms quantum.
        "mrs x10, cntfrq_el0",
        "mov x11, #200",
        "udiv x10, x10, x11",            // x10 = ticks needed for 5 ms
        "msr cntp_tval_el0, x10",

        // Preserve x0/x1 which are clobbered by the bookkeeping below.
        "stp x0, x1, [sp, #-16]!",

        // 1) Read the current core ID (Affinity Level 0).
        "mrs x0, mpidr_el1",
        "and x0, x0, #0b11",              // x0 = core_id (0..3)

        // 2) Load CURRENT_TASK_IDX base pointer.
        "ldr x1, ={current_task_idx}",

        // 3) Compute the slot for this core.
        "add x2, x1, x0, lsl #3",         // x2 = &CURRENT_TASK_IDX[core_id]

        // 4) Read the currently scheduled task index.
        "ldr x3, [x2]",                   // x3 = idx

        // 5) Move to the next task slot.
        "add x3, x3, #1",

        // 6) Clamp against the correct task table length.
        "cmp x0, #0",
        "b.eq 1f",
        // Cores 1..N -> USER_TASKS
        "mov x4, {max_user_tasks}",
        "cmp x3, x4",
        "csel x3, xzr, x3, eq",
        "b 2f",
        // Core 0 -> KERNEL_TASKS
        "1:",
        "mov x4, {max_kernel_tasks}",
        "cmp x3, x4",
        "csel x3, xzr, x3, eq",
        "2:",
        // 7) Persist the wrap-adjusted index.
        "str x3, [x2]",

        // 8) Select the right task table and branch to the function pointer.
        "cmp x0, #0",
        "b.eq 3f",
        // User/driver task dispatch
        "ldr x5, ={user_tasks}",
        "ldr x6, [x2]",
        "ldr x7, [x5, x6, lsl #3]",
        "blr x7",
        "b 4f",
        // Kernel task dispatch
        "3:",
        "ldr x5, ={kernel_tasks}",
        "ldr x6, [x2]",
        "ldr x7, [x5, x6, lsl #3]",
        "blr x7",
        "4:",
        // 9) Restore x0/x1 and exit the exception.
        "ldp x0, x1, [sp], #16",
        "eret",
        current_task_idx = sym CURRENT_TASK_IDX,
        kernel_tasks = sym KERNEL_TASKS,
        user_tasks = sym USER_TASKS,
        max_kernel_tasks = const MAX_KERNEL_TASKS,
        max_user_tasks = const MAX_USER_TASKS,
        options(noreturn)
    );
}

// === Scheduler bootstrap + call-by-call walkthrough ========================
// Minimal main.rs bring-up (runs once on each core during init):
// unsafe {
//     cpu::scheduler::install_vector_table();    // Publishes VECTOR_TABLE via VBAR_EL1; boot.s patches
//                                                // slot 0x0A0 so it branches to scheduler_irq_handler.
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
// 2) Once zero is reached, hardware raises an IRQ and indexes VBAR_EL1 + 0x0A0 (EL1h IRQ slot).
// 3) Boot.s rewrites slot 0x0A0 so the 32-bit opcode becomes `bl scheduler_irq_handler`, i.e. the
//    imm26 field is filled in with the correct offset toward the Rust handler.
// 4) Execution continues inside scheduler_irq_handler() (Rust symbol), still at EL1h.
//
// scheduler_irq_handler() flow (per instruction in the asm! block):
// 1) Re-arm cntp_tval_el0 so the next IRQ arrives 5 ms later (keeps the heartbeat going).
// 2) Save x0/x1 on the stack because the handler needs scratch registers before invoking tasks.
// 3) Read MPIDR_EL1, mask the low bits, and obtain `core_id` in x0.
// 4) Index CURRENT_TASK_IDX[core_id] (x2 points at the slot; x3 holds the previous task index).
// 5) Increment x3, wrap it if it reaches MAX_KERNEL_TASKS (core 0) or MAX_USER_TASKS (cores 1..N).
// 6) Persist the wrapped index back into CURRENT_TASK_IDX[core_id] so the next tick knows where to resume.
// 7) Select the proper task-table pointer: cores 1..N always follow USER_TASKS, core 0 sticks to
//    KERNEL_TASKS. This is enforced by the `cmp x0, #0` / branch pair, so USER_TASKS never run on core 0.
// 8) Load the function pointer (ldr x7, [table, index << 3]) and `blr x7` into the scheduled routine.
// 9) Once the task returns, restore x0/x1, execute `eret`, and the CPU resumes the interrupted context.
//
// Cores that execute USER_TASKS:
// - The handler explicitly routes any `core_id != 0` to the USER_TASKS table, so every secondary core
//   (1..MAX_CORES-1) rotates exclusively through USER_TASKS. No code changes are needed to achieve it.
// - To add more user-space routines, extend USER_TASKS and bump MAX_USER_TASKS; the scheduler logic
//   automatically honors the new length when wrapping indices.
