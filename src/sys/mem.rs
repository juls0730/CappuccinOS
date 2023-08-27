use alloc::{format, vec::Vec};
use limine::{MemmapEntry, MemoryMapEntryType};

use crate::libs::logging::{log_info, log_ok};

use super::allocator::BuddyAllocator;

static MEMMAP_REQUEST: limine::MemmapRequest = limine::MemmapRequest::new(0);

#[global_allocator]
pub static ALLOCATOR: BuddyAllocator =
    BuddyAllocator::new_unchecked(0x0010_0000 as *mut u8, 0x0008_0000);

#[derive(Clone, Copy)]
pub struct Region {
    pub usable: bool,
    pub base: usize,
    pub len: usize,
}

fn find_largest_memory_region() -> (Option<Region>, Option<Region>) {
    let memmap = MEMMAP_REQUEST.get_response().get();

    if memmap.is_none() {
        panic!("Memory map was None!");
    }

    let mut stitched_map: Vec<Region> = Vec::new();

    // pre allocate memory before adding a bunch of elements
    // This adds an additional amount of elements to the original Vector.
    // This does *not* preserve the doubling of vector that would normally happen
    // when simply .pushing as we will not modify the vector beyond the count of memmap
    // in fact, we will likely use less elements than the capacity.
    // Maybe watch this cool youtube video on the subject by Logan Smith: https://www.youtube.com/watch?v=algDLvbl1YY
    stitched_map.reserve_exact((memmap.unwrap().entry_count) as usize);
    let mut largest_region: Option<Region> = None;
    let mut second_largest_region: Option<Region> = None;

    for (i, region) in memmap.into_iter().flat_map(|a| a.memmap()).enumerate() {
        let entry = Region {
            usable: memory_section_is_usable(region),
            base: region.base as usize,
            len: region.len as usize,
        };

        if !entry.usable {
            stitched_map.push(entry);
            continue;
        }

        if i == 0 {
            largest_region = Some(entry);
            stitched_map.push(entry);
            continue;
        }

        let last_index = stitched_map.len() - 1;

        if stitched_map[last_index].usable {
            stitched_map[last_index] = Region {
                usable: true,
                base: stitched_map[last_index].base,
                len: stitched_map[last_index].len + region.len as usize,
            };

            if stitched_map[last_index].len > largest_region.map(|r| r.len).unwrap_or(0) {
                second_largest_region = largest_region;
                largest_region = Some(stitched_map[last_index]);
            } else if stitched_map[last_index].len
                > second_largest_region.map(|r| r.len).unwrap_or(0)
            {
                second_largest_region = Some(stitched_map[last_index]);
            }
        } else {
            stitched_map.push(entry);
        }
    }

    if largest_region.is_none() {
        return (None, None);
    }

    if second_largest_region.is_none() {
        return (largest_region, None);
    }

    let mut back_buffer_region = second_largest_region.unwrap();
    let mut heap_region = largest_region.unwrap();
    let min_heap_size = 0x0008_0000;

    if let Some(framebuffer_response) = crate::drivers::video::FRAMEBUFFER_REQUEST
        .get_response()
        .get()
    {
        if framebuffer_response.framebuffer_count < 1 {
            return (Some(heap_region), None);
        }

        let framebuffer = &framebuffer_response.framebuffers()[0];
        let framebuffer_size = framebuffer.height * framebuffer.pitch;

        back_buffer_region.len = framebuffer_size as usize;

        if heap_region.base == back_buffer_region.base {
            // Heap section is located at the same area as the back buffer region
            // Check if we can safely shrink the heap section
            let shrunk_heap = heap_region.len - framebuffer_size as usize;
            if (shrunk_heap) >= min_heap_size {
                heap_region.len = shrunk_heap;
                back_buffer_region.base += shrunk_heap;
            } else {
                return (Some(heap_region), None);
            }
        }

        return (Some(heap_region), Some(back_buffer_region));
    }

    return (Some(heap_region), None);
}

fn memory_section_is_usable(entry: &MemmapEntry) -> bool {
    return entry.typ == MemoryMapEntryType::Usable
        || entry.typ == MemoryMapEntryType::BootloaderReclaimable
        || entry.typ == MemoryMapEntryType::AcpiReclaimable;
}

pub fn init() {
    let (largest_region, second_largest_region) = find_largest_memory_region();

    crate::usr::tty::CONSOLE.reinit(second_largest_region);

    if largest_region.is_none() {
        panic!("Suitable memory regions not found!");
    }

    log_ok(&format!(
        "Using largest section with: {} of memory for heap at {:?}",
        (largest_region.unwrap().len),
        largest_region.unwrap().base
    ));

    log_ok(&format!(
        "Using section with: {} of memory for back buffer at {:#?}",
        (second_largest_region.unwrap().len),
        second_largest_region.unwrap().base,
    ));

    ALLOCATOR.set_heap(
        largest_region.unwrap().base as *mut u8,
        largest_region.unwrap().len as usize,
    );
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
