// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021-2022 Andre Richter <andre.o.richter@gmail.com>

//--------------------------------------------------------------------------------------------------
// Definitions
//--------------------------------------------------------------------------------------------------

// Load the address of a symbol into a register, PC-relative.
//
// The symbol must lie within +/- 4 GiB of the Program Counter.
//
// # Resources
//
// - https://sourceware.org/binutils/docs-2.36/as/AArch64_002dRelocations.html
.macro ADR_REL register, symbol
	adrp	\register, \symbol
	add	\register, \register, #:lo12:\symbol
.endm

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
.section .text._start

//------------------------------------------------------------------------------
// fn _start()
//------------------------------------------------------------------------------
_start:
	// Preserve the DTB pointer provided by the firmware (in x0).
	mov	x21, x0
	// Only proceed on the boot core. Secondary cores branch to their own entry.
	mrs	x0, MPIDR_EL1
	and	x0, x0, {CONST_CORE_ID_MASK}
	ldr	x1, BOOT_CORE_ID      // provided by bsp/__board_name__/cpu.rs
	cmp	x0, x1
	b.eq	.L_boot_core_entry
	b	.L_secondary_core_entry

.L_boot_core_entry:
	// If execution reaches here, it is the boot core.

	// Initialize DRAM.
	ADR_REL	x0, __bss_start
	ADR_REL x1, __bss_end_exclusive

.L_bss_init_loop:
	cmp	x0, x1
	b.eq	.L_prepare_rust
	stp	xzr, xzr, [x0], #16
	b	.L_bss_init_loop

	// Prepare the jump to Rust code.
.L_prepare_rust:
	// Publish the DTB pointer for Rust code.
	ADR_REL	x0, __dtb_load_addr
	str	x21, [x0]

	// Set the stack pointer.
	ADR_REL	x0, __boot_core_stack_end_exclusive
	mov	sp, x0

	// Patch the EL1h IRQ vector entry so it branches to scheduler_irq_handler.
	ADR_REL	x0, VECTOR_TABLE           // x0 = base of vector table
	add	x0, x0, #0x0A0              // x0 = address of IRQ EL1h slot
	ADR_REL	x1, scheduler_irq_handler  // x1 = scheduler handler
	sub	x2, x1, x0                  // x2 = delta (target - entry)
	asr	x2, x2, #2                  // x2 = imm26 candidate (signed)
	ubfm	x2, x2, #0, #25            // keep the low 26 bits
	movz	x3, #0x9400, lsl #16       // opcode for BL with zero offset
	orr	x2, x3, x2                  // merge opcode with imm26 payload
	str	w2, [x0]                    // store 32-bit BL instruction

	// Jump to Rust code.
	b	_start_rust

.L_secondary_core_entry:
	// Secondary cores carve out a private stack slice inside the boot stack region.
	ADR_REL	x1, __boot_core_stack_start
	ADR_REL	x2, __boot_core_stack_end_exclusive
	sub	x3, x2, x1                  // total stack span
	mov	x4, {CONST_MAX_CORES}
	udiv	x4, x3, x4                 // bytes per core
	msub	x5, x0, x4, x2             // stack_top - core_id * slice
	mov	sp, x5

	// Jump to the Rust secondary entry, passing the core_id in x0.
	b	secondary_start_rust

	// Infinitely wait for events (aka "park the core").
.L_parking_loop:
	wfe
	b	.L_parking_loop

.size	_start, . - _start
.type	_start, function
.global	_start
