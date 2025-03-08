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
// Bring to scope the macros print! and println!
mod console;
mod cpu;
mod driver;
mod panic_wait;
mod synchronization;

extern crate alloc;
//use alloc::vec::Vec;
use core::arch::asm;
use embedded_alloc::LlffHeap as Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

// Only a single core must be active and running this function.
unsafe fn kernel_init() -> ! {
    // Device Tree Base is passed as pointer in x0 at the start
    // Because is a pointer value, the type must be usize.
    let dtb: usize = unsafe {
        let mut fdt_addr: usize;
        asm!("mov {0}, x0", out(reg) fdt_addr,options(nomem, nostack)); 
        fdt_addr
    };
    
    if let Err(x) = bsp::driver::init() {
        panic!("Error initializing BSP driver subsystem: {}", x);
    }

// Initialize all device drivers.
// println! is usable from here on.
    driver::driver_manager().init_drivers();

// Transition from unsafe to safe.
    kernel_main(dtb)
}

// The main function running after the early init.
// Because dtb is a pointer value, the type must be usize.
fn kernel_main(dtb: usize) -> ! {	
    // Initialize the allocator BEFORE you use it
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(&raw mut HEAP_MEM as usize, HEAP_SIZE) }
    }
    // now the allocator is ready types like Box, Vec can be used.
    
    let numcores = cpu::get_num_cores();

    println!("RavnOS kernel");
    println!("Board; {}", bsp::board_name());
    println!("[0] Cores number: {}", numcores);
    println!("[0] Starting all cores");
    cpu::start_cores();
    println!("[0] Started all cores");
    println!("Device Tree Base (DTB) in; {:#x}", dtb);
    
    loop {
    }
}
