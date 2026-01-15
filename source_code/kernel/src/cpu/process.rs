// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Process control block definitions bridging the cooperative prototype scheduler and the
// upcoming preemptive design described in mmu-scheduler.ai.

use core::sync::atomic::{AtomicBool, Ordering};

use crate::await_kernel_uart_println;
use crate::synchronization::{interface::Mutex, NullLock};

pub const KERNEL_MAX_PROCESSES: usize = 3;
pub const USER_MAX_PROCESSES: usize = 5;

static KERNEL_PROCESS_TABLE: NullLock<KernelProcessTable> =
    NullLock::new(KernelProcessTable::new());
static USER_PROCESS_TABLE: NullLock<UserProcessTable> = NullLock::new(UserProcessTable::new());

/// All process states supported by the design document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessState {
    Created,
    Assigned,
    Running,
    Sleeping,
    WaitingSyscall,
    Zombie,
    Terminated,
}

impl Default for ProcessState {
    fn default() -> Self {
        ProcessState::Terminated
    }
}

#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct ContextFrame {
    pub gprs: [u64; 31],
    pub sp: u64,
    pub elr: u64,
    pub spsr: u64,
}

impl ContextFrame {
    pub const fn empty() -> Self {
        Self {
            gprs: [0; 31],
            sp: 0,
            elr: 0,
            spsr: 0,
        }
    }

    pub fn setup_entry(&mut self, entry: u64, sp: u64, spsr: u64) {
        self.gprs = [0; 31];
        self.sp = sp;
        self.elr = entry;
        self.spsr = spsr;
    }

    pub fn copy_from_parts(&mut self, regs: &[u64; 31], sp: u64, elr: u64, spsr: u64) {
        self.gprs.copy_from_slice(regs);
        self.sp = sp;
        self.elr = elr;
        self.spsr = spsr;
    }

    pub fn as_ptr(&self) -> *const ContextFrame {
        self as *const _
    }
}

#[repr(C, align(64))]
#[derive(Debug)]
pub struct SyscallRequest {
    pub syscall_number: u32,
    pub argc: u32,
    pub args: [u64; 6],
    pub pid: u64,
    pub timestamp_ns: u64,
}

impl SyscallRequest {
    pub const fn empty() -> Self {
        Self {
            syscall_number: 0,
            argc: 0,
            args: [0; 6],
            pid: 0,
            timestamp_ns: 0,
        }
    }
}

#[repr(C, align(64))]
#[derive(Debug)]
pub struct SyscallResponse {
    pub return_code: i64,
    pub data: [u64; 4],
    pub errno: i32,
    pub flags: u32,
}

impl SyscallResponse {
    pub const fn empty() -> Self {
        Self {
            return_code: 0,
            data: [0; 4],
            errno: 0,
            flags: 0,
        }
    }
}

/// Snapshot of an ARMv8/AArch64 context that can be parked inside an interrupt handler.
#[repr(C, align(64))]
pub struct ProcessControlBlock {
    pub pid: u64,
    pub uid: u32,
    pub gid: u32,
    pub priority: u32,
    pub state: ProcessState,
    pub claim_flag: AtomicBool,
    pub owner_core: u8,
    pub is_running: bool,
    pub pc: u64,
    pub sp: u64,
    pub lr: u64,
    pub pstate: u64,
    pub spsr_el1: u64,
    pub tpidr_el0: u64,
    pub registers: [u64; 30],
    pub sp_el0: u64,
    pub fp_registers: [u128; 32],
    pub fpcr: u64,
    pub fpsr: u64,
    pub ttbr0: u64,
    pub page_table_permissions: u32,
    pub kernel_stack_base: u64,
    pub kernel_stack_top: u64,
    pub kernel_stack_guard_page: u64,
    pub kernel_stack_chunks: u32,
    pub user_stack_base: u64,
    pub user_stack_size: usize,
    pub binary_path: [u8; 256],
    pub argv: *const *const u8,
    pub argv_kernel_copy: [u8; 1024],
    pub argc: usize,
    pub exit_code: i32,
    pub cpu_time_ms: u64,
    pub creation_time: u64,
    pub signal_pending: u64,
    pub mailbox_request: Option<SyscallRequest>,
    pub mailbox_response: Option<SyscallResponse>,
    pub context: ContextFrame,
}

unsafe impl Send for ProcessControlBlock {}

impl ProcessControlBlock {
    pub const fn new() -> Self {
        Self {
            pid: 0,
            uid: 0,
            gid: 0,
            priority: 0,
            state: ProcessState::Terminated,
            claim_flag: AtomicBool::new(false),
            owner_core: 0,
            is_running: false,
            pc: 0,
            sp: 0,
            lr: 0,
            pstate: 0,
            spsr_el1: 0,
            tpidr_el0: 0,
            registers: [0; 30],
            sp_el0: 0,
            fp_registers: [0; 32],
            fpcr: 0,
            fpsr: 0,
            ttbr0: 0,
            page_table_permissions: 0,
            kernel_stack_base: 0,
            kernel_stack_top: 0,
            kernel_stack_guard_page: 0,
            kernel_stack_chunks: 0,
            user_stack_base: 0,
            user_stack_size: 0,
            binary_path: [0; 256],
            argv: core::ptr::null(),
            argv_kernel_copy: [0; 1024],
            argc: 0,
            exit_code: 0,
            cpu_time_ms: 0,
            creation_time: 0,
            signal_pending: 0,
            mailbox_request: None,
            mailbox_response: None,
            context: ContextFrame::empty(),
        }
    }

    pub fn reset(&mut self) {
        *self = ProcessControlBlock::new();
    }

    pub fn mark_assigned(
        &mut self,
        pid: u64,
        entry: unsafe extern "C" fn(),
        priority: u32,
        ttbr0: u64,
    ) {
        self.reset();
        self.pid = pid;
        self.priority = priority;
        self.state = ProcessState::Assigned;
        self.pc = entry as u64;
        self.lr = entry as u64;
        self.is_running = false;
        self.ttbr0 = ttbr0;
        self.sp_el0 = 0;
        self.claim_flag.store(true, Ordering::Release);
    }

    pub fn mark_running(&mut self, core_id: u8) {
        self.owner_core = core_id;
        self.state = ProcessState::Running;
        self.is_running = true;
        self.claim_flag.store(true, Ordering::Release);
    }

    pub fn mark_idle(&mut self) {
        self.is_running = false;
        self.state = ProcessState::Assigned;
        self.claim_flag.store(false, Ordering::Release);
    }

    pub fn snapshot_context(&mut self, pc: u64, spsr: u64, sp_el0: u64) {
        self.pc = pc;
        self.spsr_el1 = spsr;
        self.sp_el0 = sp_el0;
    }
}

pub struct KernelProcessTable {
    entries: [ProcessControlBlock; KERNEL_MAX_PROCESSES],
}

impl KernelProcessTable {
    pub const fn new() -> Self {
        Self {
            entries: [
                ProcessControlBlock::new(),
                ProcessControlBlock::new(),
                ProcessControlBlock::new(),
            ],
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn register_kernel_task(
        &mut self,
        slot: usize,
        entry: unsafe extern "C" fn(),
        priority: u32,
        ttbr1_phys: u64,
        name: &'static str,
        stack_base: u64,
        stack_top: u64,
        initial_spsr: u64,
    ) {
        if slot >= self.entries.len() {
            await_kernel_uart_println!(
                "[process] Kernel slot {} out of range (len={})",
                slot,
                self.entries.len()
            );
            return;
        }

        let pcb = &mut self.entries[slot];
        pcb.mark_assigned((slot + 1) as u64, entry, priority, 0);
        pcb.ttbr0 = 0;
        pcb.kernel_stack_base = 0;
        pcb.kernel_stack_top = 0;
        pcb.page_table_permissions = 0;
        copy_name_into(&mut pcb.binary_path, name.as_bytes());
        pcb.mailbox_request = None;
        pcb.mailbox_response = None;
        pcb.claim_flag.store(false, Ordering::Relaxed);
        pcb.owner_core = 0;
        pcb.is_running = false;
        pcb.pstate = 0;
        // Kernel runs entirely in TTBR1, but we keep a breadcrumb for telemetry.
        pcb.kernel_stack_guard_page = ttbr1_phys;
        pcb.kernel_stack_base = stack_base;
        pcb.kernel_stack_top = stack_top;
        pcb.sp = stack_top;
        pcb.pc = entry as u64;
        pcb.lr = entry as u64;
        pcb.spsr_el1 = initial_spsr;
        pcb.context
            .setup_entry(entry as u64, stack_top, initial_spsr);
    }

    pub fn with_slot_mut<R>(
        &mut self,
        slot: usize,
        f: impl FnOnce(&mut ProcessControlBlock) -> R,
    ) -> Option<R> {
        self.entries.get_mut(slot).map(f)
    }

    pub fn with_slot<R>(
        &self,
        slot: usize,
        f: impl FnOnce(&ProcessControlBlock) -> R,
    ) -> Option<R> {
        self.entries.get(slot).map(f)
    }
}

fn copy_name_into(target: &mut [u8], name: &[u8]) {
    let len = target.len().min(name.len());
    target[..len].copy_from_slice(&name[..len]);
}

pub fn with_kernel_process_table<R>(f: impl FnOnce(&mut KernelProcessTable) -> R) -> R {
    KERNEL_PROCESS_TABLE.lock(f)
}

pub fn with_kernel_process<R>(slot: usize, f: impl FnOnce(&ProcessControlBlock) -> R) -> Option<R> {
    with_kernel_process_table(|table| table.with_slot(slot, f))
}

pub fn with_kernel_process_mut<R>(
    slot: usize,
    f: impl FnOnce(&mut ProcessControlBlock) -> R,
) -> Option<R> {
    with_kernel_process_table(|table| table.with_slot_mut(slot, f))
}

pub struct UserProcessTable {
    entries: [ProcessControlBlock; USER_MAX_PROCESSES],
}

impl UserProcessTable {
    pub const fn new() -> Self {
        Self {
            entries: [
                ProcessControlBlock::new(),
                ProcessControlBlock::new(),
                ProcessControlBlock::new(),
                ProcessControlBlock::new(),
                ProcessControlBlock::new(),
            ],
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn register_user_task(
        &mut self,
        slot: usize,
        entry: unsafe extern "C" fn(),
        priority: u32,
        ttbr0_phys: u64,
        name: &'static str,
        stack_base: u64,
        stack_top: u64,
        initial_spsr: u64,
    ) {
        if slot >= self.entries.len() {
            await_kernel_uart_println!(
                "[process] User slot {} out of range (len={})",
                slot,
                self.entries.len()
            );
            return;
        }

        let pcb = &mut self.entries[slot];
        pcb.mark_assigned(0x1000 + slot as u64, entry, priority, ttbr0_phys);
        pcb.kernel_stack_base = 0;
        pcb.kernel_stack_top = 0;
        pcb.page_table_permissions = 0;
        copy_name_into(&mut pcb.binary_path, name.as_bytes());
        pcb.mailbox_request = None;
        pcb.mailbox_response = None;
        pcb.claim_flag.store(false, Ordering::Relaxed);
        pcb.owner_core = 1;
        pcb.is_running = false;
        pcb.pstate = 0;
        pcb.user_stack_base = stack_base;
        pcb.user_stack_size = (stack_top - stack_base) as usize;
        pcb.tpidr_el0 = slot as u64;
        pcb.context
            .setup_entry(entry as u64, stack_top, initial_spsr);
        pcb.sp = stack_top;
        pcb.pc = entry as u64;
        pcb.lr = entry as u64;
        pcb.spsr_el1 = initial_spsr;
        pcb.ttbr0 = ttbr0_phys;
    }

    pub fn with_slot<R>(
        &self,
        slot: usize,
        f: impl FnOnce(&ProcessControlBlock) -> R,
    ) -> Option<R> {
        self.entries.get(slot).map(f)
    }

    pub fn with_slot_mut<R>(
        &mut self,
        slot: usize,
        f: impl FnOnce(&mut ProcessControlBlock) -> R,
    ) -> Option<R> {
        self.entries.get_mut(slot).map(f)
    }
}

pub fn with_user_process_table<R>(f: impl FnOnce(&mut UserProcessTable) -> R) -> R {
    USER_PROCESS_TABLE.lock(f)
}

pub fn with_user_process<R>(slot: usize, f: impl FnOnce(&ProcessControlBlock) -> R) -> Option<R> {
    USER_PROCESS_TABLE.lock(|table| table.with_slot(slot, f))
}

pub fn with_user_process_mut<R>(
    slot: usize,
    f: impl FnOnce(&mut ProcessControlBlock) -> R,
) -> Option<R> {
    USER_PROCESS_TABLE.lock(|table| table.with_slot_mut(slot, f))
}
