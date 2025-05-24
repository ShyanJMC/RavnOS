mod boot;
use core::ptr::{write_volatile};
// Maybe you are asking; why are you importin macro println! if it
// is defined in console/mod.rs ? Not should be first "use crate::console"
// and then the macro is automatically imported? 
// Well, not. Because when is imported in main.rs as module "mod console.rs",
// is enabled as global macro in the hole program as root crate, and because
// of that you just need use "use crate::println".
// Remember that this is because is a macro, is not the same behaviour in functions.
use crate::println;


use aarch64_cpu::asm;
use aarch64_cpu::asm::barrier::SY;
// Data Synchronization Barrier
use aarch64_cpu::asm::barrier::dsb;
// Instruction Synchronization Barrier
use aarch64_cpu::asm::barrier::isb;
// Send EVent
use aarch64_cpu::asm::sev;

/// Pause execution on the core.
#[inline(always)]
pub fn wait_forever() -> ! {
    loop {
        asm::wfe()
    }
}

use aarch64_cpu::registers::Readable;

// Get the number of cores based in MPIDR_EL1 registry
pub fn get_num_cores() -> u8 {
	let mut num_cores = 1; // Contamos el núcleo 0 por defecto
	let max_aff_level = 3; // Máximo nivel de afinidad (depende del sistema)
	for aff in 0..max_aff_level {
		let affinity_shift = aff * 8;
		let mpidr_value = aarch64_cpu::registers::MPIDR_EL1.get();
		for i in 0..(1 << 8) {
			if ((mpidr_value >> affinity_shift) & 0xff) == i {
				num_cores += 1;
			}
		}
	}
    num_cores
}

// Start all cores available in the SOC
pub fn start_cores() {
    const CORE_START_ADDR: u64 = 0x80000;
    const MAILBOX_BASE: u64 = 0x4000_0000;
    let num_cores = get_num_cores();

    for i in 1..num_cores {
        let mb = MAILBOX_BASE + (i as u64) * 0x10;
        println!("[0] Starting core {} with total MAILBOX; {}", &i, &mb);
        println!("[0] Setting Spin Table for core {} with address {}", &i, &CORE_START_ADDR);
        unsafe {
            write_volatile(mb as *mut u64, CORE_START_ADDR);
            dsb(SY);
            isb(SY);
            write_volatile((MAILBOX_BASE + 0x8 + (i as u64) * 8) as *mut u64, 1);
            sev();
        }
        println!("[0] Core {} started", i);
    }
}
