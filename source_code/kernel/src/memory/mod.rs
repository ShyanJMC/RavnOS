// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Memory subsystem entry point: page allocator + 3-level 64 KiB tables.

pub mod mmu;
pub mod page_allocator;

use alloc::vec;
use alloc::vec::Vec;

use crate::await_kernel_uart_println;
use crate::bsp::DtbSummary;
use crate::cpu::scheduler;
use crate::synchronization::interface::Mutex;
use crate::synchronization::NullLock;

use mmu::{AccessPermissions, KernelTables, MemoryType, MmuError, PageAttributes, Shareability};
use page_allocator::{PageAllocator, RamRegion, ReservationKind, ReservedRegion, PAGE_SIZE};

const EMERGENCY_POOL_BYTES: u64 = 10 * 1024 * 1024;
const DEFAULT_MMIO_LENGTH: u64 = 0x0200_0000; // 32 MiB of device mappings.

static MEMORY_MANAGER: NullLock<Option<MemoryManager>> = NullLock::new(None);

#[derive(Clone, Debug)]
pub struct MemorySummary {
    pub granule_size: usize,
    pub ram_regions: Vec<RamRegion>,
    pub reserved_regions: Vec<ReservedRegion>,
    pub kernel_image: (u64, u64),
    pub emergency_pool: Option<ReservedRegion>,
    pub total_free_bytes: u64,
    pub mmio_base: u64,
}

pub struct MemoryManager {
    regions: Vec<RamRegion>,
    allocator: PageAllocator,
    kernel_tables: KernelTables,
    user_spaces: Vec<UserAddressSpace>,
    mmio_base: u64,
    kernel_image_span: (u64, u64),
    emergency_pool: Option<ReservedRegion>,
}

impl MemoryManager {
    fn build(summary: &DtbSummary) -> Self {
        let regions = collect_regions(summary);
        let mut allocator = PageAllocator::from_regions(&regions);

        let (kernel_start, kernel_end) = kernel_image_range();
        let kernel_size = kernel_end - kernel_start;
        allocator.reserve_span(kernel_start, kernel_size, ReservationKind::KernelImage);

        let emergency_pages =
            ((EMERGENCY_POOL_BYTES + (PAGE_SIZE as u64) - 1) / (PAGE_SIZE as u64)) as usize;
        let emergency_span =
            allocator.allocate_contiguous(emergency_pages, ReservationKind::EmergencyPool);
        if emergency_span.is_none() {
            await_kernel_uart_println!(
                "[mem] WARNING: unable to reserve emergency pool of {} bytes",
                EMERGENCY_POOL_BYTES
            );
        }

        let mut tables = KernelTables::new();

        for region in &regions {
            let start = align_down(region.start);
            let end = align_up(region.start + region.size);
            if end <= start {
                continue;
            }
            let size = end - start;
            tables
                .map_identity(
                    start,
                    size,
                    PageAttributes::new(
                        MemoryType::Normal,
                        Shareability::InnerShareable,
                        AccessPermissions::KernelReadWrite,
                        false,
                    ),
                )
                .expect("Failed to map DRAM region");
        }

        let mmio_base = summary.peripherals.mmio_start;
        let kernel_mmio_windows = peripheral_mmio_windows(&[
            summary.peripherals.mmio_start,
            summary.peripherals.uart_pl011,
            summary.peripherals.gpio,
            summary.peripherals.spi0,
            summary.peripherals.gic_distributor,
            summary.peripherals.gic_redistributor,
            summary.peripherals.local_intc,
        ]);
        for window_base in kernel_mmio_windows {
            match tables.map_identity(
                window_base,
                DEFAULT_MMIO_LENGTH,
                PageAttributes::new(
                    MemoryType::Device,
                    Shareability::InnerShareable,
                    AccessPermissions::KernelReadWrite,
                    true,
                ),
            ) {
                Ok(()) => {}
                Err(MmuError::AlreadyMapped(addr)) => await_kernel_uart_println!(
                    "[mem] Skipping kernel MMIO window @ {:#x} (already mapped, first duplicate page {:#x})",
                    window_base,
                    addr
                ),
                Err(err) => panic!("[mem] Failed to map kernel peripheral MMIO window: {:?}", err),
            }
        }

        await_kernel_uart_println!(
            "[mem] Kernel image {kernel_start:#x} -> reserved {kernel_size} bytes"
        );
        await_kernel_uart_println!(
            "[mem] Free RAM after reservations: {} bytes",
            allocator.total_free_bytes()
        );

        let user_mmio_windows = peripheral_mmio_windows(&[
            summary.peripherals.mmio_start,
            summary.peripherals.uart_pl011,
            summary.peripherals.gpio,
            summary.peripherals.spi0,
            summary.peripherals.gic_distributor,
            summary.peripherals.gic_redistributor,
            summary.peripherals.local_intc,
        ]);
        let mut user_spaces = Vec::with_capacity(scheduler::MAX_USER_TASKS);
        for _ in 0..scheduler::MAX_USER_TASKS {
            user_spaces.push(UserAddressSpace::new(&regions, &user_mmio_windows));
        }

        for (idx, space) in user_spaces.iter().enumerate() {
            scheduler::set_user_task_ttbr(idx, space.ttbr0_phys());
        }

        Self {
            regions,
            allocator,
            kernel_tables: tables,
            user_spaces,
            mmio_base,
            kernel_image_span: (kernel_start, kernel_end),
            emergency_pool: emergency_span,
        }
    }

    pub fn kernel_tables(&self) -> &KernelTables {
        &self.kernel_tables
    }

    pub fn total_free_bytes(&self) -> u64 {
        self.allocator.total_free_bytes()
    }

    pub fn user_space_ttbr0(&self, idx: usize) -> Option<u64> {
        self.user_spaces.get(idx).map(|space| space.ttbr0_phys())
    }

    fn snapshot(&self) -> MemorySummary {
        MemorySummary {
            granule_size: PAGE_SIZE,
            ram_regions: self.regions.clone(),
            reserved_regions: self.allocator.reserved_regions().to_vec(),
            kernel_image: self.kernel_image_span,
            emergency_pool: self.emergency_pool.clone(),
            total_free_bytes: self.allocator.total_free_bytes(),
            mmio_base: self.mmio_base,
        }
    }

    fn debug_identity_checks(&self) {
        let (image_start, image_end) = self.kernel_image_span;
        self.debug_identity_check("kernel_start", image_start);
        if image_end > image_start {
            self.debug_identity_check("kernel_end", image_end - 1);
        }

        let (stack_start, stack_end) = boot_stack_range();
        self.debug_identity_check("boot_stack_start", stack_start);
        self.debug_identity_check("boot_stack_end", stack_end);

        self.kernel_tables.dump_mapping(image_start + 0x6480); // roughly where early printk lives
        self.kernel_tables.dump_mapping(0x200);
    }

    fn debug_identity_check(&self, label: &str, addr: u64) {
        match self.kernel_tables.translate(addr) {
            Some(phys) => await_kernel_uart_println!(
                "[mem][verify] {label}: virt {:#x} -> phys {:#x}",
                addr,
                phys
            ),
            None => await_kernel_uart_println!(
                "[mem][verify] {label}: virt {:#x} unmapped in kernel tables",
                addr
            ),
        }
    }
}

pub fn init(summary: Option<&DtbSummary>) {
    let manager = if let Some(s) = summary {
        MemoryManager::build(s)
    } else {
        let fallback = DtbSummary::fallback();
        MemoryManager::build(&fallback)
    };
    manager.debug_identity_checks();
    MEMORY_MANAGER.lock(|slot: &mut Option<MemoryManager>| {
        *slot = Some(manager);
    });

    await_kernel_uart_println!(
        "[mem] Page tables root @ {:#x}, MAIR_EL1 = {:#x}",
        kernel_ttbr1_phys().unwrap_or(0),
        mmu::mair::value()
    );
}

pub fn kernel_ttbr1_phys() -> Option<u64> {
    MEMORY_MANAGER.lock(|slot: &mut Option<MemoryManager>| {
        slot.as_ref()
            .map(|mgr: &MemoryManager| mgr.kernel_tables().root_phys())
    })
}

pub fn total_free_bytes() -> Option<u64> {
    MEMORY_MANAGER.lock(|slot: &mut Option<MemoryManager>| {
        slot.as_ref()
            .map(|mgr: &MemoryManager| mgr.total_free_bytes())
    })
}

pub fn summary() -> Option<MemorySummary> {
    MEMORY_MANAGER.lock(|slot: &mut Option<MemoryManager>| {
        slot.as_ref().map(|mgr: &MemoryManager| mgr.snapshot())
    })
}
pub fn enable_mmu_on_this_core() -> Result<(), &'static str> {
    let ttbr1 = kernel_ttbr1_phys().ok_or("Kernel translation tables not initialized")?;
    unsafe { mmu::enable_kernel_mmu(ttbr1) }
}

struct UserAddressSpace {
    tables: KernelTables,
}

impl UserAddressSpace {
    fn new(regions: &[RamRegion], mmio_windows: &[u64]) -> Self {
        let mut tables = KernelTables::new();

        for region in regions {
            let start = align_down(region.start);
            let end = align_up(region.start + region.size);
            if end <= start {
                continue;
            }
            let size = end - start;
            tables
                .map_identity(
                    start,
                    size,
                    PageAttributes::new(
                        MemoryType::Normal,
                        Shareability::InnerShareable,
                        AccessPermissions::UserReadWrite,
                        false,
                    ),
                )
                .expect("Failed to map shared user DRAM region");
        }

        for &window_base in mmio_windows {
            match tables.map_identity(
                window_base,
                DEFAULT_MMIO_LENGTH,
                PageAttributes::new(
                    MemoryType::Device,
                    Shareability::InnerShareable,
                    AccessPermissions::UserReadWrite,
                    true,
                ),
            ) {
                Ok(()) => {}
                Err(MmuError::AlreadyMapped(addr)) => await_kernel_uart_println!(
                    "[mem] Skipping user MMIO window @ {:#x} (already mapped, first duplicate page {:#x})",
                    window_base,
                    addr
                ),
                Err(err) => panic!("[mem] Failed to map shared user MMIO window: {:?}", err),
            }
        }

        Self { tables }
    }

    fn ttbr0_phys(&self) -> u64 {
        self.tables.root_phys()
    }
}

fn collect_regions(summary: &DtbSummary) -> Vec<RamRegion> {
    if summary.memory_regions.is_empty() {
        return vec![RamRegion::new(0, 512 * 1024 * 1024)];
    }

    summary
        .memory_regions
        .iter()
        .map(|region| RamRegion::new(region.start, region.size))
        .collect()
}

fn peripheral_mmio_windows(candidates: &[u64]) -> Vec<u64> {
    let mut aligned: Vec<u64> = candidates
        .iter()
        .copied()
        .filter(|addr| *addr != 0)
        .map(|addr| align_down_len(addr, DEFAULT_MMIO_LENGTH))
        .collect();
    aligned.sort_unstable();

    let mut windows = Vec::new();
    for base in aligned {
        if let Some(&prev) = windows.last() {
            if base < prev + DEFAULT_MMIO_LENGTH {
                continue;
            }
        }
        windows.push(base);
    }

    windows
}

extern "C" {
    static __boot_core_stack_start: u8;
    static __boot_core_stack_end_exclusive: u8;
}

fn kernel_image_range() -> (u64, u64) {
    extern "C" {
        static __rpi_phys_binary_load_addr: u8;
        static __bss_end_exclusive: u8;
    }

    unsafe {
        (
            &__rpi_phys_binary_load_addr as *const u8 as u64,
            &__bss_end_exclusive as *const u8 as u64,
        )
    }
}

fn boot_stack_range() -> (u64, u64) {
    unsafe {
        (
            &__boot_core_stack_start as *const u8 as u64,
            &__boot_core_stack_end_exclusive as *const u8 as u64,
        )
    }
}

#[inline(always)]
fn align_down(value: u64) -> u64 {
    let mask = (PAGE_SIZE as u64) - 1;
    value & !mask
}

#[inline(always)]
fn align_up(value: u64) -> u64 {
    let mask = (PAGE_SIZE as u64) - 1;
    if value & mask == 0 {
        value
    } else {
        (value & !mask) + (PAGE_SIZE as u64)
    }
}

#[inline(always)]
fn align_down_len(value: u64, len: u64) -> u64 {
    debug_assert!(len.is_power_of_two());
    value & !(len - 1)
}
