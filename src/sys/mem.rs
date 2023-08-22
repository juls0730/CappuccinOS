use alloc::format;
use limine::{
    MemmapEntry,
    MemoryMapEntryType::{BootloaderReclaimable, Usable},
    NonNullPtr,
};

use crate::{libs::logging::log_ok, print, println};

use super::allocator::BuddyAllocator;

static MEMMAP_REQUEST: limine::MemmapRequest = limine::MemmapRequest::new(0);

#[global_allocator]
pub static ALLOCATOR: BuddyAllocator =
    BuddyAllocator::new_unchecked(0x0010_0000 as *mut u8, 0x0008_0000);

fn find_largest_memory_region() -> Option<&'static NonNullPtr<MemmapEntry>> {
    let memmap = MEMMAP_REQUEST.get_response().get();

    if memmap.is_none() {
        return None;
    }

    let mut largest_region: Option<&NonNullPtr<MemmapEntry>> = None;
    for region in memmap.into_iter().flat_map(|a| a.memmap()) {
        if region.typ != Usable && region.typ != BootloaderReclaimable {
            continue;
        }

        let current_region = region;

        if largest_region.is_none() {
            largest_region = Some(current_region);
            continue;
        }

        if region.len > largest_region.unwrap().len {
            largest_region = Some(current_region);
        }
    }

    return largest_region;
}

pub fn memory_map_info() {
    let memmap = MEMMAP_REQUEST.get_response().get();

    println!("====== Memory Map ======");
    for (i, region) in memmap.into_iter().flat_map(|a| a.memmap()).enumerate() {
        let (size, label) = label_units((region.len) as usize);
        println!(
            "Entry {:<2}: {:#018x} - {:#018x}; {:<9} Type: {:?}",
            i,
            region.base,
            region.len + region.base,
            format!("{size}{label};"),
            region.typ
        );
    }
}

pub fn init() {
    let largest_region = find_largest_memory_region().expect("Failed to retrieve usable memory!");

    log_ok(&format!(
        "Using largest section with: {} of memory",
        (largest_region.len)
    ));

    ALLOCATOR.set_heap(largest_region.base as *mut u8, largest_region.len as usize);
}

pub fn label_units(bytes: usize) -> (usize, &'static str) {
    if bytes >> 30 > 0 {
        return (bytes >> 30, "GiB");
    } else if bytes >> 20 > 0 {
        return (bytes >> 20, "MiB");
    } else if bytes >> 10 > 0 {
        return (bytes >> 10, "KiB");
    } else {
        return (bytes, "Bytes");
    }
}
