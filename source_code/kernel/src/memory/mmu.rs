// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Minimal 3-level page table builder for 64 KiB granules targeting TTBR1_EL1.

use alloc::boxed::Box;
use alloc::vec::Vec;

use super::page_allocator::PAGE_SIZE;
use crate::await_kernel_uart_println;
use aarch64_cpu::asm::barrier;
use aarch64_cpu::registers::{
    ID_AA64MMFR0_EL1, MAIR_EL1, SCTLR_EL1, TCR_EL1, TTBR0_EL1, TTBR1_EL1,
};
use core::arch::asm;
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

const ENTRIES_PER_TABLE: usize = PAGE_SIZE / core::mem::size_of::<u64>(); // 8192 entries.
const PAGE_SHIFT: u64 = 16;
const LEVEL_BITS: u64 = 13;
const LEVEL_MASK: u64 = (1 << LEVEL_BITS) - 1;

const DESC_VALID: u64 = 1;
const DESC_TYPE: u64 = 1 << 1;
const DESC_AF: u64 = 1 << 10;
const DESC_SH_SHIFT: u64 = 8;
const DESC_AP_SHIFT: u64 = 6;
const DESC_ATTR_SHIFT: u64 = 2;
const DESC_PXN: u64 = 1 << 53;
const DESC_UXN: u64 = 1 << 54;

#[repr(C, align(65536))]
pub struct PageTable {
    entries: [u64; ENTRIES_PER_TABLE],
}

impl PageTable {
    pub const fn new() -> Self {
        Self {
            entries: [0; ENTRIES_PER_TABLE],
        }
    }

    fn entry_mut(&mut self, index: usize) -> &mut u64 {
        &mut self.entries[index]
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MemoryType {
    Normal,
    Device,
}

#[derive(Clone, Copy, Debug)]
pub enum Shareability {
    NonShareable,
    InnerShareable,
}

#[derive(Clone, Copy, Debug)]
pub enum AccessPermissions {
    KernelReadWrite,
    KernelReadOnly,
    UserReadWrite,
    UserReadOnly,
}

#[derive(Clone, Copy, Debug)]
pub struct PageAttributes {
    pub mem_type: MemoryType,
    pub shareability: Shareability,
    pub access: AccessPermissions,
    pub execute_never: bool,
}

impl PageAttributes {
    pub const fn new(
        mem_type: MemoryType,
        shareability: Shareability,
        access: AccessPermissions,
        execute_never: bool,
    ) -> Self {
        Self {
            mem_type,
            shareability,
            access,
            execute_never,
        }
    }

    fn attr_index(&self) -> u64 {
        match self.mem_type {
            MemoryType::Normal => mair::NORMAL_INDEX,
            MemoryType::Device => mair::DEVICE_INDEX,
        }
    }

    fn shareability_bits(&self) -> u64 {
        match self.shareability {
            Shareability::NonShareable => 0b00,
            Shareability::InnerShareable => 0b11,
        }
    }

    fn access_bits(&self) -> u64 {
        match self.access {
            AccessPermissions::KernelReadWrite => 0b00,
            AccessPermissions::KernelReadOnly => 0b10,
            AccessPermissions::UserReadWrite => 0b01,
            AccessPermissions::UserReadOnly => 0b11,
        }
    }
}

impl Default for PageAttributes {
    fn default() -> Self {
        Self::new(
            MemoryType::Normal,
            Shareability::InnerShareable,
            AccessPermissions::KernelReadWrite,
            false,
        )
    }
}

#[derive(Debug)]
pub enum MmuError {
    AlreadyMapped(u64),
    UnalignedAddress(u64),
    UnalignedSize(u64),
}

pub mod mair {
    pub const DEVICE_INDEX: u64 = 0;
    pub const NORMAL_INDEX: u64 = 1;

    const DEVICE_NGNRE: u64 = 0x04;
    const NORMAL_WB: u64 = 0xFF;

    pub const fn value() -> u64 {
        (NORMAL_WB << (NORMAL_INDEX * 8)) | (DEVICE_NGNRE << (DEVICE_INDEX * 8))
    }
}

pub struct KernelTables {
    root: Box<PageTable>,
    owned: Vec<Box<PageTable>>,
}

impl KernelTables {
    pub fn new() -> Self {
        Self {
            root: Box::new(PageTable::new()),
            owned: Vec::new(),
        }
    }

    #[inline]
    pub fn root_phys(&self) -> u64 {
        self.root.as_ref() as *const _ as u64
    }

    pub fn translate(&self, virt: u64) -> Option<u64> {
        let l1_idx = level_index(virt, 2);
        let l2_idx = level_index(virt, 1);
        let l3_idx = level_index(virt, 0);

        let l1_entry = *self.root.entries.get(l1_idx)?;
        let l2_table_ptr = Self::entry_table(l1_entry)?;
        let l2_table = unsafe { &*l2_table_ptr };
        let l2_entry = *l2_table.entries.get(l2_idx)?;
        let l3_table_ptr = Self::entry_table(l2_entry)?;
        let l3_table = unsafe { &*l3_table_ptr };
        let l3_entry = *l3_table.entries.get(l3_idx)?;

        if (l3_entry & DESC_VALID) == 0 {
            return None;
        }

        let phys_base = l3_entry & !((PAGE_SIZE as u64) - 1);
        let offset = virt & ((PAGE_SIZE as u64) - 1);
        Some(phys_base + offset)
    }

    pub fn dump_mapping(&self, virt: u64) {
        let l1_idx = level_index(virt, 2);
        let l2_idx = level_index(virt, 1);
        let l3_idx = level_index(virt, 0);

        let l1_entry = self.root.entries.get(l1_idx).copied().unwrap_or(0);
        await_kernel_uart_println!(
            "[mmu][dump] VA {:#x} L1 idx {} entry {:#x}",
            virt,
            l1_idx,
            l1_entry
        );
        if (l1_entry & DESC_VALID) == 0 {
            return;
        }

        let l2_table = unsafe { &*Self::entry_table(l1_entry).unwrap() };
        let l2_entry = l2_table.entries.get(l2_idx).copied().unwrap_or(0);
        await_kernel_uart_println!(
            "[mmu][dump] VA {:#x} L2 idx {} entry {:#x}",
            virt,
            l2_idx,
            l2_entry
        );
        if (l2_entry & DESC_VALID) == 0 {
            return;
        }

        let l3_table = unsafe { &*Self::entry_table(l2_entry).unwrap() };
        let l3_entry = l3_table.entries.get(l3_idx).copied().unwrap_or(0);
        await_kernel_uart_println!(
            "[mmu][dump] VA {:#x} L3 idx {} entry {:#x}",
            virt,
            l3_idx,
            l3_entry
        );
    }

    fn entry_table(entry: u64) -> Option<*const PageTable> {
        if (entry & DESC_VALID) == 0 || (entry & DESC_TYPE) == 0 {
            return None;
        }
        let phys = entry & !((PAGE_SIZE as u64) - 1);
        Some(phys as *const PageTable)
    }

    pub fn map_identity(
        &mut self,
        phys_start: u64,
        size: u64,
        attrs: PageAttributes,
    ) -> Result<(), MmuError> {
        self.map_range(phys_start, phys_start, size, attrs)
    }

    pub fn map_range(
        &mut self,
        virt_start: u64,
        phys_start: u64,
        size: u64,
        attrs: PageAttributes,
    ) -> Result<(), MmuError> {
        if virt_start & ((PAGE_SIZE as u64) - 1) != 0 {
            return Err(MmuError::UnalignedAddress(virt_start));
        }
        if phys_start & ((PAGE_SIZE as u64) - 1) != 0 {
            return Err(MmuError::UnalignedAddress(phys_start));
        }
        if size & ((PAGE_SIZE as u64) - 1) != 0 {
            return Err(MmuError::UnalignedSize(size));
        }

        let page_count = size / (PAGE_SIZE as u64);

        for page in 0..page_count {
            let vaddr = virt_start + page * (PAGE_SIZE as u64);
            let paddr = phys_start + page * (PAGE_SIZE as u64);
            self.map_single_page(vaddr, paddr, attrs)?;
        }

        Ok(())
    }

    fn map_single_page(
        &mut self,
        virt_addr: u64,
        phys_addr: u64,
        attrs: PageAttributes,
    ) -> Result<(), MmuError> {
        let l1_idx = level_index(virt_addr, 2);
        let l2_idx = level_index(virt_addr, 1);
        let l3_idx = level_index(virt_addr, 0);

        let l1_entry_ptr = self.root.entry_mut(l1_idx) as *mut u64;
        let l2_table_ptr = self.ensure_child_table(l1_entry_ptr);
        let l2_entry_ptr = unsafe { (*l2_table_ptr).entry_mut(l2_idx) as *mut u64 };
        let l3_table_ptr = self.ensure_child_table(l2_entry_ptr);
        let l3_entry = unsafe { &mut *(*l3_table_ptr).entry_mut(l3_idx) };

        if (*l3_entry & DESC_VALID) != 0 {
            return Err(MmuError::AlreadyMapped(virt_addr));
        }

        *l3_entry = build_page_descriptor(phys_addr, attrs);
        Ok(())
    }

    fn ensure_child_table(&mut self, entry: *mut u64) -> *mut PageTable {
        unsafe {
            if (*entry & DESC_VALID) == 0 {
                self.owned.push(Box::new(PageTable::new()));
                let idx = self.owned.len() - 1;
                let table_ptr = self.owned[idx].as_mut() as *mut PageTable;
                let phys = table_ptr as u64;
                *entry = (phys & !((PAGE_SIZE as u64) - 1)) | DESC_TYPE | DESC_VALID;
                table_ptr
            } else {
                let phys = *entry & !((PAGE_SIZE as u64) - 1);
                phys as *mut PageTable
            }
        }
    }
}

fn level_index(addr: u64, level: usize) -> usize {
    let shift = PAGE_SHIFT + level as u64 * LEVEL_BITS;
    ((addr >> shift) & LEVEL_MASK) as usize
}

fn build_page_descriptor(phys_addr: u64, attrs: PageAttributes) -> u64 {
    let mut desc = (phys_addr & !((PAGE_SIZE as u64) - 1)) | DESC_VALID | DESC_TYPE | DESC_AF;
    desc |= attrs.attr_index() << DESC_ATTR_SHIFT;
    desc |= attrs.shareability_bits() << DESC_SH_SHIFT;
    desc |= attrs.access_bits() << DESC_AP_SHIFT;

    if attrs.execute_never {
        desc |= DESC_PXN | DESC_UXN;
    }

    desc
}

/// Program TTBRx, MAIR, TCR and enable the MMU/caches for the current core.
pub unsafe fn enable_kernel_mmu(ttbr_phys: u64) -> Result<(), &'static str> {
    if ttbr_phys & ((PAGE_SIZE as u64) - 1) != 0 {
        return Err("TTBR base address not 64 KiB aligned");
    }

    await_kernel_uart_println!("[mmu] enable_kernel_mmu: tables_phys={:#x}", ttbr_phys);
    if SCTLR_EL1.matches_all(SCTLR_EL1::M::Enable) {
        await_kernel_uart_println!("[mmu] MMU already enabled; skipping reconfigure");
        return Ok(());
    }

    if !ID_AA64MMFR0_EL1.matches_all(ID_AA64MMFR0_EL1::TGran64::Supported) {
        return Err("CPU does not support the 64 KiB translation granule");
    }

    // Program attribute encodings: Attr0=device, Attr1=cacheable normal memory.
    let mair_value = mair::value();
    MAIR_EL1.set(mair_value);
    await_kernel_uart_println!("[mmu] MAIR programmed -> 0x{:016x}", mair_value);

    // Publish the same root for both TTBRs so we can transition smoothly while still
    // experimenting with user spaces that expect TTBR0 slots.
    TTBR0_EL1.set_baddr(ttbr_phys);
    TTBR1_EL1.set_baddr(ttbr_phys);
    await_kernel_uart_println!(
        "[mmu] TTBR0_EL1.baddr=0x{:016x} TTBR1_EL1.baddr=0x{:016x}",
        TTBR0_EL1.get_baddr(),
        TTBR1_EL1.get_baddr()
    );

    let t0sz = (64 - 48) as u64;
    let t1sz = (64 - 48) as u64;
    let tcr_value = TCR_EL1::TBI0::Used
        + TCR_EL1::IPS::Bits_40
        + TCR_EL1::TG0::KiB_64
        + TCR_EL1::SH0::Inner
        + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::EPD0::EnableTTBR0Walks
        + TCR_EL1::A1::TTBR0
        + TCR_EL1::T0SZ.val(t0sz)
        + TCR_EL1::TBI1::Used
        + TCR_EL1::TG1::KiB_64
        + TCR_EL1::SH1::Inner
        + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::EPD1::EnableTTBR1Walks
        + TCR_EL1::T1SZ.val(t1sz);
    TCR_EL1.write(tcr_value);
    await_kernel_uart_println!("[mmu] TCR_EL1 set -> 0x{:016x}", TCR_EL1.get());

    barrier::dsb(barrier::ISH);
    barrier::isb(barrier::SY);

    // Ensure caches and TLBs are clean before turning on the MMU and caches.
    unsafe {
        asm!("ic iallu", options(nostack, preserves_flags));
    }
    barrier::dsb(barrier::ISH);
    barrier::isb(barrier::SY);
    unsafe {
        asm!("tlbi vmalle1", options(nostack, preserves_flags));
    }
    barrier::dsb(barrier::ISH);
    barrier::isb(barrier::SY);

    await_kernel_uart_println!("[mmu] enabling SCTLR.M (enabling caches)");
    SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
    barrier::dsb(barrier::ISH);
    barrier::isb(barrier::SY);
    await_kernel_uart_println!(
        "[mmu] SCTLR_EL1 now 0x{:016x} (M={},C={},I={})",
        SCTLR_EL1.get(),
        SCTLR_EL1.matches_all(SCTLR_EL1::M::Enable),
        SCTLR_EL1.matches_all(SCTLR_EL1::C::Cacheable),
        SCTLR_EL1.matches_all(SCTLR_EL1::I::Cacheable)
    );
    Ok(())
}
