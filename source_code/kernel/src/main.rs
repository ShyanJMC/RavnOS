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

// Memory alloc
extern crate alloc;
use embedded_alloc::LlffHeap as Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

const HEAP_SIZE: usize = 128 * 1024;

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
    uart_println!("[0] Starting all cores");
    bsp::start_secondary_cores(core_count);
    uart_println!("[0] Started all cores");

    if let Some(summary) = dtb_info {
        if !summary.entries.is_empty() {
            for entry in summary.entries {
                uart_println!("[1] DTB data; {entry}");
            }
            uart_println!("[1] End reading DTB");
        }
    }

    loop {}
}

fn init_heap() {
    use core::mem::MaybeUninit;

    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    // SAFETY: Called exactly once during kernel startup before concurrency is introduced.
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
}
