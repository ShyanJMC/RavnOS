use fdt::Fdt;
use core::slice;
use core::ptr;

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;

use crate::println;

fn get_dtb_size(dtb_addr: usize) -> usize {
    let dtb_ptr = dtb_addr as *const u32;
    u32::from_be(unsafe { *dtb_ptr.add(1) }) as usize
}

pub fn parse_dtb(dtb_addr: usize) -> Option<Vec<String>> {
    let mut some_vec: Vec<String> = Vec::new();
    let dtb_magic = unsafe { u32::from_be(*(dtb_addr as *const u32)) };
    if dtb_magic != 0xd00dfeed {
        println!("Bad DTB magic: {:#x}", dtb_magic);
        return None;
    }

    let dtb_size = get_dtb_size(dtb_addr);
    let dtb_slice = unsafe { slice::from_raw_parts(dtb_addr as *const u8, dtb_size) };

    let fdt = match Fdt::new(dtb_slice) {
        Ok(fdt) => fdt,
        Err(e) => {
            println!("Error FDT: {:?}", e);
            return None;
        }
    };

    println!("DTB found: version {}", fdt.total_size());

    // Root mode
    let root = fdt.root();
    let soc = fdt.find_node("/soc");
    println!("[INFO] SOC detected; {}", if soc.is_some() { "yes" } else { "no" });
    if let Some(soc) = soc {
        for child in soc.children() {
            some_vec.push(format!("[DTB INFO]: System on a CHIP (SOC) name; {}", child.name));
        }
    }
    
    println!("[DTB INFO]: Root model {}", root.model());
    println!("[DTB INFO]: CPUS number {}", fdt.cpus().count());
    println!("[DTB INFO]: Memory regions {}", fdt.memory().regions().count());
    println!("[DTB INFO]: Memory regions start at {}", fdt.memory().regions().next().unwrap().starting_address as usize);
    println!("[DTB INFO]: Bootargs; {:?}", fdt.chosen().bootargs());
    println!("[DTB INFO]: standard output (stdout); {:?}", fdt.chosen().stdout());
    println!("[DTB INFO]: standard output (stdout); {:?}", fdt.chosen().stdin());
    
    return Some(some_vec);
}