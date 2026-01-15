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
	bl	el2_to_el1
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

	// Patch all EL1 IRQ vector entries so they branch to scheduler_irq_handler.
	ADR_REL	x0, VECTOR_TABLE           // x0 = base of vector table
	ADR_REL	x1, scheduler_irq_handler  // x1 = scheduler handler
	movz	x3, #0x9400, lsl #16       // opcode for BL with zero offset
	mov	x4, #0                      // slot index counter (0..3)
	mov	x5, #0x80                   // first IRQ slot offset (EL1t)
	mov	x6, #0x200                  // stride between IRQ slots

.L_patch_next_irq_slot:
	// slot_offsets = 0x080 (EL1t IRQ), 0x280 (EL1h IRQ), 0x480 (Lower EL AArch64 IRQ), 0x680 (Lower EL AArch32 IRQ)
	cmp	x4, #4
	b.eq	.L_patch_done
	madd	x7, x4, x6, x5             // offset = 0x80 + slot * 0x200
	add	x8, x0, x7
	sub	x2, x1, x8                  // x2 = delta (target - entry)
	asr	x2, x2, #2                  // x2 = imm26 candidate (signed)
	ubfm	x2, x2, #0, #25            // keep the low 26 bits
	orr	x2, x3, x2                  // merge opcode with imm26 payload
	str	w2, [x8]                    // store 32-bit BL instruction
	add	x4, x4, #1
	b	.L_patch_next_irq_slot

.L_patch_done:
	dsb	sy                         // ensure store is visible before cache maintenance
	isb
	ic	iallu                      // flush I-cache so CPUs don't execute stale slot
	dsb	sy
	isb

	// Jump to Rust code.
	b	_start_rust

.L_secondary_core_entry:
	bl	el2_to_el1
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

.global	el2_to_el1
el2_to_el1:
	mrs	x9, CurrentEL
	lsr	x9, x9, #2
	cmp	x9, #2
	b.ne	2f
	// Allow EL1 access to the physical and virtual timers/counters.
	mrs	x10, CNTHCTL_EL2
	orr	x10, x10, #(1 << 0)        // EL1PCTEN
	orr	x10, x10, #(1 << 1)        // EL1PCEN
	orr	x10, x10, #(1 << 3)        // EL1VCTEN
	msr	CNTHCTL_EL2, x10
	mov	x10, xzr
	msr	CNTVOFF_EL2, x10
	msr	CNTHP_CTL_EL2, x10
	// Switch to AArch64 EL1h with interrupts masked.
	mov	x10, #(1 << 31)            // HCR_EL2.RW = EL1 AArch64
	msr	HCR_EL2, x10
	mov	x10, xzr
	msr	CPTR_EL2, x10
	mov	x10, #(0b0101)
	mov	x11, #(0b1111 << 6)
	orr	x10, x10, x11
	msr	SPSR_EL2, x10
	adr	x10, 1f
	msr	ELR_EL2, x10
	eret
1:
2:
	ret

.size	_start, . - _start
.type	_start, function
.global	_start
