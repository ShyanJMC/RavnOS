mod boot;
use aarch64_cpu::asm;

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
	let CORE_START_ADDR: u64 = 0x80000;
	let MAILBOX_BASE: u64 = 0x4000_0000;
    let num_cores = get_num_cores();

    for i in 1..num_cores {
        let mailbox_offset = (i as u64) * 0x10;
        let mailbox_addr = MAILBOX_BASE + mailbox_offset;
        
        unsafe {
        	// Here the inmutability is disabled because of that "unsafe"
        	// Escribir la dirección de arranque en el mailbox
            *(mailbox_addr as *mut u64) = CORE_START_ADDR;
            // Start core
            let release_addr = MAILBOX_BASE + 0x8 + (i as u64 * 8);
            *(release_addr as *mut u32) = 0x1;
            aarch64_cpu::asm::sev(); // Send Event
        }
    }
}
