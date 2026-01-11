// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021-2023 Andre Richter <andre.o.richter@gmail.com>

//! Architectural boot code.
//!
//! # Orientation
//!
//! Since arch modules are imported into generic modules using the path attribute, the path of this
//! file is:
//!
//! crate::cpu::boot::arch_boot

use crate::uart_println;
use core::arch::global_asm;

// Assembly counterpart to this file.
global_asm!(
    include_str!("boot.s"),
    CONST_CORE_ID_MASK = const 0b11,
    CONST_MAX_CORES = const super::scheduler::MAX_CORES
);

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// The Rust entry of the `kernel` binary.
///
/// The function is called from the assembly `_start` function.
#[no_mangle]
pub unsafe extern "C" fn _start_rust() -> ! {
    crate::kernel_init()
}

/// Secondary-core entry invoked from the assembly trampoline once a non-boot CPU is released.
#[no_mangle]
pub unsafe extern "C" fn secondary_start_rust(core_id: u64) -> ! {
    uart_println!(
        "[{}] secondary_start_rust(): entering secondary Rust path at EL1",
        core_id
    );
    crate::secondary_core_main(core_id as usize)
}
