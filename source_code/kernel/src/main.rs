#![allow(clippy::upper_case_acronyms)]
#![feature(asm_const)]
#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![feature(trait_alias)]
#![no_main]
#![no_std]

//! RavnOS kernel
/// This kernel is designed to be used in ARM64 bits

#[cfg(not(target_arch = "aarch64"))]
compile_error!("RavnOS is only available for aarch64");

mod bsp;
mod console;
mod cpu;
mod driver;
mod panic_wait;
mod print;
mod synchronization;

/// Only a single core must be active and running this function.
unsafe fn kernel_init() -> ! {
    if let Err(x) = bsp::driver::init() {
        panic!("Error initializing BSP driver subsystem: {}", x);
    }

/// Initialize all device drivers.
/// println! is usable from here on.
    driver::driver_manager().init_drivers();

/// Transition from unsafe to safe.
    kernel_main()
}

/// The main function running after the early init.
fn kernel_main() -> ! {	
    let numcores = cpu::get_num_cores();

    println!("RavnOS kernel");
    println!("[0] Cores number: {}", numcores);
    println!("[0] Starting all cores");
    cpu::start_cores();
    println!("[0] Started all cores");
    loop {
    }
}
