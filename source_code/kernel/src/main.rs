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

use bsp::raspberrypi::dtb;

// Memory alloc
extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
//use crate::alloc::string::ToString;
//use alloc::format;
//use core::arch::asm;
use embedded_alloc::LlffHeap as Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

// Only a single core must be active and running this function.
unsafe fn kernel_init() -> ! {  
    if let Err(x) = bsp::driver::init() {
        panic!("Error initializing BSP driver subsystem: {}", x);
    }

// Initialize all device drivers.
// println! is usable from here on.
    driver::driver_manager().init_drivers();

// Transition from unsafe to safe.
    //kernel_main(dtb)
    kernel_main()
}

// The main function running after the early init.
fn kernel_main() -> ! {	
    // Initialize the allocator BEFORE you use it
    {
        use core::mem::MaybeUninit;
        // 128 * 1024 = 128 KiB of memory allocated
        const HEAP_SIZE: usize = 128 * 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        //unsafe { HEAP.init(&raw mut HEAP_MEM as usize, HEAP_SIZE) }
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }
    // now the allocator is ready types like Box, Vec can be used.
    
    let numcores = cpu::get_num_cores();

    println!("RavnOS kernel");
    println!("Board; {}", bsp::board_name());
    println!("[0] Cores number: {}", numcores);
    println!("[0] Starting all cores");
    cpu::start_cores();
    println!("[0] Started all cores");
    
    // Remember that 0x0x80000 is RavnOS kernel isself when
    // arm_64bit=1 is enabled.
    // DTB must have 0xd00dfeed as magic number in Big Endian
    let mut dtb_data: Vec<String> = Vec::new();
    println!("[0] Verifying DTB at; {:x}",read_u32_be(0x000000000000033c));
    if print_mem_magic(0x000000000000033c) == 0xd00dfeed {
        println!("[0] DTB found.");
    	if let Some(data) = dtb::parse_dtb(0x000000000000033c){
    	    dtb_data = data;
    	};
    }
    
    if !dtb_data.is_empty(){
        for i in dtb_data {
            println!("[1] DTB data; {i}");
        }
        println!("[1] End reading DTB");
    }
    
    loop {
    }
}

fn print_mem_magic(mem_address: usize) -> u32 {
    let magic = unsafe { core::ptr::read_volatile(mem_address as *const u32) };
    magic.to_be() // Convert Little Endian (used by Raspberry Pi) to Big Endian (used here by RavnOS)
}

fn read_u32_be(addr: usize) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const u32).to_be() }
}