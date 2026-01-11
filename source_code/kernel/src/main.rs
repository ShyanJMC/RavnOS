#![allow(clippy::upper_case_acronyms)]
#![feature(format_args_nl)]
#![feature(trait_alias)]
#![no_main]
#![no_std]

//! RavnOS kernel
// This kernel is designed to be used in ARM64 bits
#[cfg(not(target_arch = "aarch64"))]
compile_error!("RavnOS is only available for aarch64");

mod bsp;
mod critical_section_impl;
// Bring to scope the UART logging macros.
mod console;
mod cpu;
mod panic_wait;
mod synchronization;

use aarch64_cpu::asm;
use core::hint::spin_loop;
use core::sync::atomic::{AtomicBool, Ordering};
use cpu::{kernel_threads, scheduler, userdebug_threads};

// Memory alloc
extern crate alloc;
use embedded_alloc::LlffHeap as Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

const HEAP_SIZE: usize = 128 * 1024;
const SECONDARY_SPIN_LIMIT: usize = 1_000_000;

static SCHEDULER_READY: AtomicBool = AtomicBool::new(false);
static CORE_ONLINE: [AtomicBool; scheduler::MAX_CORES] =
    [const { AtomicBool::new(false) }; scheduler::MAX_CORES];

// Only a single core must be active and running this function.
unsafe fn kernel_init() -> ! {
    init_heap();

    if let Err(x) = bsp::init() {
        panic!("Error initializing BSP driver subsystem: {}", x);
    }

    // Transition from unsafe to safe.
    kernel_main()
}

// The main function running after the early init.
fn kernel_main() -> ! {
    uart_println!("RavnOS kernel");
    uart_println!("Board; {}", bsp::board_name());

    let dtb_info = bsp::probe_dtb();
    let core_count = dtb_info
        .as_ref()
        .and_then(|summary| summary.cpu_count)
        .unwrap_or_else(|| bsp::default_core_count());

    uart_println!("[0] Cores number: {}", core_count);
    if core_count > 1 {
        uart_println!("[0] Starting secondary cores (1..{})", core_count - 1);
        for core_id in 1..core_count {
            bsp::start_secondary_core(core_id);
        }
        uart_println!("[0] Secondary core start requests issued");
        wait_for_secondary_online(core_count);
    } else {
        uart_println!("[0] Single-core system detected; no secondary cores to start");
    }

    if let Some(summary) = dtb_info {
        if !summary.entries.is_empty() {
            for entry in summary.entries {
                uart_println!("[1] DTB data; {entry}");
            }
            uart_println!("[1] End reading DTB");
        }
    }

    kernel_init_scheduler();

    loop {}
}

fn init_heap() {
    use core::mem::MaybeUninit;

    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    // SAFETY: Called exactly once during kernel startup before concurrency is introduced.
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
}

fn kernel_init_scheduler() {
    // SAFETY: Scheduler globals are singletons per core; this runs once on the boot core.
    unsafe {
        scheduler::install_vector_table();
        scheduler::setup_generic_timer_5ms();
        scheduler::enable_irq();
    }
    SCHEDULER_READY.store(true, Ordering::Release);
    asm::sev();
    kernel_threads::run_debug_checks();
    uart_println!("[0] Scheduler armed: vector table installed, timer running, IRQs enabled");
}

pub fn secondary_core_main(core_id: usize) -> ! {
    uart_println!(
        "[{}] Secondary core online; waiting for scheduler init on core 0",
        core_id
    );
    mark_secondary_online(core_id);
    while !SCHEDULER_READY.load(Ordering::Acquire) {
        spin_loop();
    }

    unsafe {
        scheduler::install_vector_table();
        scheduler::setup_generic_timer_5ms();
        scheduler::enable_irq();
    }
    uart_println!(
        "[{}] Scheduler armed on secondary core: timer running, IRQs enabled",
        core_id
    );
    userdebug_threads::run_debug_checks();

    loop {
        asm::wfi();
    }
}

fn mark_secondary_online(core_id: usize) {
    if core_id < CORE_ONLINE.len() {
        CORE_ONLINE[core_id].store(true, Ordering::Release);
    }
}

fn wait_for_secondary_online(core_count: usize) {
    for core_id in 1..core_count {
        let mut spins = 0;
        while spins < SECONDARY_SPIN_LIMIT {
            if CORE_ONLINE[core_id].load(Ordering::Acquire) {
                uart_println!("[0] Secondary core {} acknowledged startup", core_id);
                break;
            }
            spin_loop();
            spins += 1;
        }

        if !CORE_ONLINE[core_id].load(Ordering::Acquire) {
            uart_println!(
                "[0] WARNING: core {} never signaled online state (likely unsupported in this environment)",
                core_id
            );
        }
    }
}
