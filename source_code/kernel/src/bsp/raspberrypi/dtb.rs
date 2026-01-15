use core::convert::TryInto;
use core::mem::MaybeUninit;
use core::ptr::read_volatile;
use core::slice;
use core::sync::atomic::{AtomicBool, Ordering};
use fdt::node::FdtNode;
use fdt::standard_nodes::{Aliases, Cpu};
use fdt::Fdt;

extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::await_kernel_uart_println;

/// Magic value expected at the start of a flattened device tree blob.
pub const MAGIC: u32 = 0xd00dfeed;

const FALLBACK_DTB_ADDR: usize = 0x0000_0000_0000_033c;
const FALLBACK_RAM_SIZE_BYTES: u64 = 1024 * 1024 * 1024; // 1 GiB

#[no_mangle]
#[link_section = ".text._start_arguments"]
static mut __dtb_load_addr: u64 = 0;

/// High level view of the parsed DTB that the rest of the kernel cares about.
#[derive(Clone, Debug)]
pub struct Summary {
    pub entries: Vec<String>,
    pub cpu_count: Option<usize>,
    pub peripherals: PeripheralsLayout,
    pub model: String,
    pub compatibles: Vec<String>,
    pub memory_regions: Vec<MemoryRegion>,
    pub cpu_release_addrs: Vec<u64>,
}

/// Simple description of a RAM range reported by the firmware.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MemoryRegion {
    pub start: u64,
    pub size: u64,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct PeripheralsLayout {
    pub mmio_start: u64,
    pub uart_pl011: u64,
    pub gpio: u64,
    pub spi0: u64,
    pub gic_distributor: u64,
    pub gic_redistributor: u64,
    pub local_intc: u64,
}

static DTB_READY: AtomicBool = AtomicBool::new(false);
static mut DTB_SUMMARY: MaybeUninit<Summary> = MaybeUninit::uninit();

impl Summary {
    pub fn fallback() -> Self {
        #[cfg(any(feature = "bsp_rpi4", feature = "bsp_qemu"))]
        {
            Self {
                entries: Vec::new(),
                cpu_count: Some(4),
                peripherals: PeripheralsLayout {
                    mmio_start: 0xFE20_0000,
                    uart_pl011: 0xFE20_1000,
                    gpio: 0xFE20_0000,
                    gic_distributor: 0x4004_1000,
                    gic_redistributor: 0x4004_2000,
                    local_intc: 0x4000_0000,
                    ..Default::default()
                },
                model: "Raspberry Pi 4 (fallback)".into(),
                compatibles: vec!["raspberrypi,4-fallback".into()],
                memory_regions: vec![MemoryRegion {
                    start: 0,
                    size: FALLBACK_RAM_SIZE_BYTES,
                }],
                cpu_release_addrs: vec![0xD8, 0xE0, 0xE8, 0xF0],
            }
        }
        #[cfg(all(
            not(any(feature = "bsp_rpi4", feature = "bsp_qemu")),
            feature = "bsp_rpi5"
        ))]
        {
            Self {
                entries: Vec::new(),
                cpu_count: Some(4),
                peripherals: PeripheralsLayout::default(),
                model: "Raspberry Pi 5 (fallback)".into(),
                compatibles: vec!["raspberrypi,5-fallback".into()],
                memory_regions: vec![MemoryRegion {
                    start: 0,
                    size: FALLBACK_RAM_SIZE_BYTES,
                }],
                cpu_release_addrs: Vec::new(),
            }
        }
    }
}

fn cached_summary() -> Option<&'static Summary> {
    if DTB_READY.load(Ordering::Acquire) {
        // SAFETY: Once the atomic is set, the summary has been initialized.
        Some(unsafe { DTB_SUMMARY.assume_init_ref() })
    } else {
        None
    }
}

pub fn load_addr() -> usize {
    let addr = unsafe { __dtb_load_addr as usize };
    if addr != 0 {
        addr
    } else {
        FALLBACK_DTB_ADDR
    }
}

fn store_summary(summary: Summary) -> &'static Summary {
    if let Some(existing) = cached_summary() {
        return existing;
    }

    // SAFETY: single-writer during early boot.
    unsafe {
        DTB_SUMMARY.write(summary);
        DTB_READY.store(true, Ordering::Release);
        DTB_SUMMARY.assume_init_ref()
    }
}

fn dtb_size(dtb_addr: usize) -> usize {
    let dtb_ptr = dtb_addr as *const u32;
    // SAFETY: The caller ensures that `dtb_addr` points to a memory mapped DTB.
    u32::from_be(unsafe { *dtb_ptr.add(1) }) as usize
}

fn read_u32_be(addr: usize) -> u32 {
    unsafe { read_volatile(addr as *const u32).to_be() }
}

/// Verify that the firmware provided DTB looks valid and parse it into a summary.
pub fn probe() -> Option<Summary> {
    match ensure_loaded() {
        Ok(summary) => Some(summary.clone()),
        Err(error) => {
            await_kernel_uart_println!("[0] Failed to parse DTB: {}", error);
            None
        }
    }
}

pub fn ensure_loaded() -> Result<&'static Summary, &'static str> {
    if let Some(summary) = cached_summary() {
        return Ok(summary);
    }

    let primary_addr = load_addr();
    let mut selected_addr = None;

    let mut probe_addr = |addr: usize| {
        let magic = read_u32_be(addr);
        await_kernel_uart_println!("[0] Verifying DTB at {addr:#x}; magic {magic:#x}");
        if magic == MAGIC {
            selected_addr = Some(addr);
            true
        } else {
            await_kernel_uart_println!("[0] DTB not found at {addr:#x} (bad magic: {magic:#x})");
            false
        }
    };

    if !probe_addr(primary_addr) {
        if primary_addr != FALLBACK_DTB_ADDR {
            probe_addr(FALLBACK_DTB_ADDR);
        }
    }

    let dtb_addr = selected_addr.ok_or("DTB not present")?;
    await_kernel_uart_println!("[0] DTB found at {dtb_addr:#x}.");

    let summary = parse(dtb_addr)?;
    Ok(store_summary(summary))
}

pub fn peripherals_layout() -> Option<&'static PeripheralsLayout> {
    cached_summary().map(|summary| &summary.peripherals)
}

pub fn memory_regions() -> Option<&'static [MemoryRegion]> {
    cached_summary().map(|summary| summary.memory_regions.as_slice())
}

pub fn cpu_release_addrs() -> Option<&'static [u64]> {
    cached_summary().map(|summary| summary.cpu_release_addrs.as_slice())
}

pub fn parse(dtb_addr: usize) -> Result<Summary, &'static str> {
    let dtb_size = dtb_size(dtb_addr);
    // SAFETY: The caller guarantees that the DTB is resident in memory.
    let dtb_slice = unsafe { slice::from_raw_parts(dtb_addr as *const u8, dtb_size) };

    let fdt = match Fdt::new(dtb_slice) {
        Ok(fdt) => fdt,
        Err(err) => {
            await_kernel_uart_println!("Error FDT: {:?}", err);
            return Err("Failed to decode DTB");
        }
    };

    await_kernel_uart_println!("DTB found: version {}", fdt.total_size());

    let mut entries = Vec::new();
    let root = fdt.root();
    let soc = fdt.find_node("/soc");
    await_kernel_uart_println!(
        "[INFO] SOC detected; {}",
        if soc.is_some() { "yes" } else { "no" }
    );

    if let Some(soc) = soc {
        for child in soc.children() {
            entries.push(format!(
                "[DTB INFO]: System on a CHIP (SOC) name; {}",
                child.name
            ));
        }
    }

    let model = root.model().to_string();
    await_kernel_uart_println!("[DTB INFO]: Root model {}", model);

    let compatibles = root
        .compatible()
        .all()
        .map(|entry| entry.to_string())
        .collect::<Vec<_>>();

    let (cpu_count, cpu_release_addrs) = collect_cpu_release_info(&fdt);
    await_kernel_uart_println!("[DTB INFO]: CPUS number {}", cpu_count);

    let mut memory_regions = Vec::new();
    for region in fdt.memory().regions() {
        let start = region.starting_address as u64;
        let size = region.size.unwrap_or(0) as u64;

        if size == 0 {
            continue;
        }

        memory_regions.push(MemoryRegion { start, size });
        await_kernel_uart_println!(
            "[DTB INFO]: Memory region start {start:#x} size {size:#x} ({}) bytes",
            size
        );
    }
    if memory_regions.is_empty() {
        await_kernel_uart_println!("[DTB INFO]: Memory regions missing from DTB");
    }

    await_kernel_uart_println!("[DTB INFO]: Bootargs; {:?}", fdt.chosen().bootargs());
    await_kernel_uart_println!(
        "[DTB INFO]: standard output (stdout); {:?}",
        fdt.chosen().stdout()
    );
    await_kernel_uart_println!(
        "[DTB INFO]: standard input (stdin); {:?}",
        fdt.chosen().stdin()
    );

    let peripherals = parse_peripherals(&fdt)?;

    Ok(Summary {
        entries,
        cpu_count: Some(cpu_count),
        peripherals,
        model,
        compatibles,
        memory_regions,
        cpu_release_addrs,
    })
}

fn parse_peripherals(fdt: &Fdt<'_>) -> Result<PeripheralsLayout, &'static str> {
    let aliases = fdt.aliases();

    let uart_pl011 = normalize_peripheral_addr(
        resolve_pl011_from_alias(&aliases, "serial1")
            .or_else(|| resolve_pl011_from_alias(&aliases, "serial0"))
            .or_else(|| find_compatible_address(fdt, &["arm,pl011"]))
            .ok_or("PL011 UART node missing in DTB")?,
    );

    let gpio = normalize_peripheral_addr(
        resolve_alias_address(&aliases, "gpio")
            .or_else(|| find_compatible_address(fdt, &["brcm,bcm2835-gpio", "brcm,bcm2711-gpio"]))
            .ok_or("GPIO node missing in DTB")?,
    );

    let spi0 = normalize_peripheral_addr(
        resolve_alias_address(&aliases, "spi0")
            .or_else(|| find_named_child_address(fdt, "spi@7e204000"))
            .or_else(|| find_named_child_address(fdt, "spi@7d204000"))
            .or_else(|| find_compatible_address(fdt, &["brcm,bcm2835-spi"]))
            .ok_or("SPI0 node missing in DTB")?,
    );

    let gic_node = find_compatible_node(fdt, &["arm,gic-400"]).ok_or("GIC-400 node missing")?;
    let gic_distributor = normalize_peripheral_addr(
        node_reg_entry(gic_node, 0).ok_or("GIC distributor reg missing")?,
    );
    let gic_redistributor = normalize_peripheral_addr(node_reg_entry(gic_node, 1).unwrap_or(0));

    let local_intc = normalize_peripheral_addr(
        find_compatible_node(fdt, &["brcm,bcm2836-l1-intc", "brcm,l2-intc"])
            .and_then(|node| node_reg_entry(node, 0))
            .ok_or("Local interrupt controller node missing")?,
    );

    let mmio_start = [
        uart_pl011,
        gpio,
        spi0,
        gic_distributor,
        gic_redistributor,
        local_intc,
    ]
    .into_iter()
    .filter(|addr| *addr != 0)
    .min()
    .unwrap_or(0);

    Ok(PeripheralsLayout {
        mmio_start,
        uart_pl011,
        gpio,
        spi0,
        gic_distributor,
        gic_redistributor,
        local_intc,
    })
}

fn resolve_alias_address<'a>(aliases: &Option<Aliases<'_, 'a>>, alias: &str) -> Option<u64> {
    aliases
        .as_ref()
        .and_then(|entries| entries.resolve_node(alias))
        .and_then(node_first_reg_u64)
}

fn resolve_pl011_from_alias<'a>(aliases: &Option<Aliases<'_, 'a>>, alias: &str) -> Option<u64> {
    aliases.as_ref().and_then(|entries| {
        let node = entries.resolve_node(alias)?;
        if node_is_pl011(&node) {
            node_first_reg_u64(node)
        } else {
            None
        }
    })
}

fn node_is_pl011(node: &FdtNode<'_, '_>) -> bool {
    node.compatible()
        .map(|list| list.all().any(|entry| entry == "arm,pl011"))
        .unwrap_or(false)
}

fn normalize_peripheral_addr(addr: u64) -> u64 {
    const VC_BUS_BASE: u64 = 0x7E00_0000;
    const VC_BUS_SPAN: u64 = 0x0200_0000;
    const VC_PHYS_BASE_PI4: u64 = 0xFE00_0000;

    const LOCAL_BUS_BASE: u64 = 0x4000_0000;
    const LOCAL_BUS_SPAN: u64 = 0x0100_0000;
    const LOCAL_PHYS_BASE_PI4: u64 = 0xFF80_0000;

    if (VC_BUS_BASE..VC_BUS_BASE + VC_BUS_SPAN).contains(&addr) {
        VC_PHYS_BASE_PI4 + (addr - VC_BUS_BASE)
    } else if (LOCAL_BUS_BASE..LOCAL_BUS_BASE + LOCAL_BUS_SPAN).contains(&addr) {
        LOCAL_PHYS_BASE_PI4 + (addr - LOCAL_BUS_BASE)
    } else {
        addr
    }
}

fn find_named_child_address(fdt: &Fdt<'_>, path: &str) -> Option<u64> {
    fdt.find_node(path).and_then(node_first_reg_u64)
}

fn find_compatible_address(fdt: &Fdt<'_>, compatibles: &[&str]) -> Option<u64> {
    find_compatible_node(fdt, compatibles).and_then(node_first_reg_u64)
}

fn find_compatible_node<'fdt, 'dtb>(
    fdt: &'fdt Fdt<'dtb>,
    compatibles: &[&str],
) -> Option<FdtNode<'fdt, 'dtb>> {
    fdt.all_nodes().find(|node| {
        node.compatible()
            .map(|list| {
                list.all()
                    .any(|entry| compatibles.iter().any(|c| *c == entry))
            })
            .unwrap_or(false)
    })
}

fn node_first_reg_u64(node: FdtNode<'_, '_>) -> Option<u64> {
    node_reg_entry(node, 0)
}

fn node_reg_entry(node: FdtNode<'_, '_>, index: usize) -> Option<u64> {
    let mut regs = node.reg()?;
    regs.nth(index)
        .map(|region| region.starting_address as usize as u64)
}

fn collect_cpu_release_info(fdt: &Fdt<'_>) -> (usize, Vec<u64>) {
    let mut cpu_count = 0usize;
    let mut release_slots = Vec::new();

    for cpu in fdt.cpus() {
        cpu_count += 1;
        let core_id = cpu.ids().first();

        if release_slots.len() <= core_id {
            release_slots.resize(core_id + 1, 0);
        }

        match cpu_release_addr(&cpu) {
            Some(addr) => {
                release_slots[core_id] = addr;
                await_kernel_uart_println!(
                    "[DTB INFO]: cpu{} release address {:#x}",
                    core_id,
                    addr
                );
            }
            None => await_kernel_uart_println!(
                "[0] WARNING: cpu{} missing cpu-release-addr property",
                core_id
            ),
        }
    }

    (cpu_count, release_slots)
}

fn cpu_release_addr(cpu: &Cpu<'_, '_>) -> Option<u64> {
    cpu.property("cpu-release-addr")
        .and_then(|prop| be_bytes_to_u64(prop.value))
}

fn be_bytes_to_u64(bytes: &[u8]) -> Option<u64> {
    match bytes.len() {
        8 => bytes.try_into().ok().map(u64::from_be_bytes),
        4 => bytes
            .try_into()
            .ok()
            .map(|raw: [u8; 4]| u32::from_be_bytes(raw) as u64),
        _ => None,
    }
}
