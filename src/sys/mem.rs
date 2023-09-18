use core::{arch::asm, fmt::Display};

use alloc::vec::Vec;
use limine::{MemmapEntry, MemoryMapEntryType, NonNullPtr};

use super::allocator::BuddyAllocator;

static MEMMAP_REQUEST: limine::MemmapRequest = limine::MemmapRequest::new(0);

#[global_allocator]
pub static ALLOCATOR: BuddyAllocator =
    BuddyAllocator::new_unchecked(0x0001_0000 as *mut u8, 0x0008_0000);

// pub struct Region {
//     pub usable: bool,
//     pub base: usize,
//     pub len: usize,
// }

// impl Display for Region {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         write!(
//             f,
//             "[ {:#018X?} ] Usabele: {}",
//             self.base..self.base + self.len,
//             self.usable
//         )
//     }
// }

// fn find_largest_memory_region() -> (Option<Region>, Option<Region>) {
//     let memmap = MEMMAP_REQUEST.get_response().get();

//     if memmap.is_none() {
//         panic!("Memory map was None!");
//     }

//     let mut stitched_map: Vec<Region> = Vec::new();

//     // pre allocate memory before adding a bunch of elements
//     // This adds an additional amount of elements to the original Vector.
//     // This does *not* preserve the doubling of vector that would normally happen
//     // when simply .pushing as we will not modify the vector beyond the count of memmap
//     // in fact, we will likely use less elements than the capacity.
//     // Maybe watch this cool youtube video on the subject by Logan Smith: https://www.youtube.com/watch?v=algDLvbl1YY
//     stitched_map.reserve_exact((memmap.unwrap().entry_count) as usize);
//     let mut largest_region: Option<Region> = None;
//     let mut second_largest_region: Option<Region> = None;

//     for (i, region) in memmap.into_iter().flat_map(|a| a.memmap()).enumerate() {
//         let entry = Region {
//             usable: memory_section_is_usable(region),
//             base: region.base as usize,
//             len: region.len as usize,
//         };

//         #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
//         if region.typ == limine::MemoryMapEntryType::Framebuffer {
//             crate::arch::set_mtrr(
//                 region.base,
//                 region.len,
//                 crate::arch::MTRRMode::WriteCombining,
//             );
//         }

//         if !entry.usable {
//             stitched_map.push(entry);
//             continue;
//         }

//         if i == 0 {
//             largest_region = Some(entry);
//             stitched_map.push(entry);
//             continue;
//         }

//         let last_index = stitched_map.len() - 1;

//         if stitched_map[last_index].usable {
//             stitched_map[last_index] = Region {
//                 usable: true,
//                 base: stitched_map[last_index].base,
//                 len: stitched_map[last_index].len + region.len as usize,
//             };

//             if stitched_map[last_index].len > largest_region.map(|r| r.len).unwrap_or(0) {
//                 second_largest_region = largest_region;
//                 largest_region = Some(stitched_map[last_index]);
//             } else if stitched_map[last_index].len
//                 > second_largest_region.map(|r| r.len).unwrap_or(0)
//             {
//                 second_largest_region = Some(stitched_map[last_index]);
//             }
//         } else {
//             stitched_map.push(entry);
//         }
//     }

//     if largest_region.is_none() {
//         return (None, None);
//     }

//     if second_largest_region.is_none() {
//         return (largest_region, None);
//     }

//     let mut back_buffer_region = second_largest_region.unwrap();
//     let mut heap_region = largest_region.unwrap();
//     let min_heap_size = 0x0008_0000;

//     let framebuffer_response = crate::drivers::video::FRAMEBUFFER_REQUEST
//         .get_response()
//         .get();

//     if framebuffer_response.is_none() {
//         return (Some(heap_region), None);
//     }

//     // eww, variable redeclaration
//     let framebuffer_response = framebuffer_response.unwrap();
//     if framebuffer_response.framebuffer_count < 1 {
//         return (Some(heap_region), None);
//     }

//     let framebuffer = &framebuffer_response.framebuffers()[0];
//     let framebuffer_size = framebuffer.height * framebuffer.pitch;

//     back_buffer_region.len = framebuffer_size as usize;

//     if heap_region.base == back_buffer_region.base {
//         // Heap section is located at the same area as the back buffer region
//         // Check if we can safely shrink the heap section
//         let shrunk_heap = heap_region.len - framebuffer_size as usize;
//         if (shrunk_heap) >= min_heap_size {
//             heap_region.len = shrunk_heap;
//             back_buffer_region.base += shrunk_heap;
//         } else {
//             return (Some(heap_region), None);
//         }
//     }

//     return (Some(heap_region), Some(back_buffer_region));
// }

fn stitch_memory_map(memmap: &mut [NonNullPtr<MemmapEntry>]) -> &mut [NonNullPtr<MemmapEntry>] {
    let mut null_index_ptr = 0;

    for i in 1..memmap.len() {
        let region = &memmap[i];

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if region.typ == limine::MemoryMapEntryType::Framebuffer {
            crate::arch::set_mtrr(
                region.base,
                region.len,
                crate::arch::MTRRMode::WriteCombining,
            );
        }

        if !memory_section_is_usable(&memmap[i]) {
            memmap[null_index_ptr].len = memmap[i].len;
            memmap[null_index_ptr].base = memmap[i].base;
            memmap[null_index_ptr].typ = memmap[i].typ;

            null_index_ptr += 1;
            continue;
        }

        if null_index_ptr > 0 && memory_section_is_usable(&memmap[null_index_ptr - 1]) {
            memmap[null_index_ptr - 1].len += memmap[i].len;

            continue;
        }

        if memory_section_is_usable(&memmap[i - 1]) {
            memmap[i - 1].len += memmap[i].len;

            memmap[null_index_ptr].len = memmap[i - 1].len;
            memmap[null_index_ptr].base = memmap[i - 1].base;
            memmap[null_index_ptr].typ = memmap[i - 1].typ;

            null_index_ptr += 1;
            continue;
        }
    }

    return &mut memmap[0..null_index_ptr];
}

fn find_largest_memory_regions(
    memmap: &[NonNullPtr<MemmapEntry>],
    min_heap_size: usize,
) -> (
    Option<&NonNullPtr<MemmapEntry>>,
    Option<&NonNullPtr<MemmapEntry>>,
) {
    let mut largest_region: Option<&NonNullPtr<MemmapEntry>> = None;
    let mut second_largest_region: Option<&NonNullPtr<MemmapEntry>> = None;
    let mut framebuffer_region: Option<&NonNullPtr<MemmapEntry>> = None;

    for region in memmap.iter() {
        if region.typ == limine::MemoryMapEntryType::Framebuffer {
            framebuffer_region = Some(region);
            continue;
        }

        if !memory_section_is_usable(region) {
            continue;
        }

        if largest_region.is_none() || region.len > largest_region.unwrap().len {
            second_largest_region = largest_region;
            largest_region = Some(region);
        } else if second_largest_region.is_none() || region.len > second_largest_region.unwrap().len
        {
            second_largest_region = Some(region);
        }
    }

    // TODO: Resize second largest region if it's not big enough to fit a framebuffer
    if framebuffer_region.is_none() || second_largest_region.is_none() {
        return (largest_region, None);
    }

    let framebuffer_size = framebuffer_region.unwrap().len;

    // crate::println!("{framebuffer_size}");

    if second_largest_region.unwrap().len >= framebuffer_size {
        return (largest_region, second_largest_region);
    }

    // let shrunk_heap = largest_region.unwrap().len - framebuffer_size;

    // if shrunk_heap < min_heap_size as u64 {
    //     return (largest_region, None);
    // }

    // unsafe {
    //     (*second_largest_region.unwrap().as_ptr()).base = largest_region.unwrap().base;

    //     (*largest_region.unwrap().as_ptr()).len = shrunk_heap;
    //     (*second_largest_region.unwrap().as_ptr()).base += shrunk_heap;
    // }

    return (largest_region, None);
}

#[inline]
fn memory_section_is_usable(entry: &MemmapEntry) -> bool {
    return entry.typ == MemoryMapEntryType::Usable
        || entry.typ == MemoryMapEntryType::BootloaderReclaimable
        || entry.typ == MemoryMapEntryType::AcpiReclaimable;
}

pub fn init() {
    let memmap_request = MEMMAP_REQUEST.get_response().get_mut();
    if memmap_request.is_none() {
        panic!("Memory map was None!");
    }

    let mut memmap = core::mem::ManuallyDrop::new(memmap_request.unwrap());
    let memmap = stitch_memory_map(memmap.memmap_mut());

    let (largest_region, second_largest_region) = find_largest_memory_regions(&memmap, 0x0008_0000);

    crate::usr::tty::CONSOLE.reinit(second_largest_region);

    crate::println!("====== Memory Map ======");
    for entry in memmap.iter() {
        crate::println!(
            "[ {:#018X?} ] Type: {:?} Size: {:?}",
            entry.base..entry.base + entry.len,
            entry.typ,
            entry.len as usize
        )
    }

    if largest_region.is_none() {
        panic!("Suitable memory regions not found!");
    }

    ALLOCATOR.set_heap(
        largest_region.unwrap().base as *mut u8,
        largest_region.unwrap().len as usize,
    );

    crate::log_ok!(
        "Using largest section with: {} bytes of memory for heap at {:#X}",
        largest_region.unwrap().len,
        largest_region.unwrap().base
    );

    if second_largest_region.is_some() {
        crate::log_ok!(
        		"Using second largest section with: {} bytes of memory for framebuffer mirroring at {:#X}",
        		second_largest_region.unwrap().len,
        		second_largest_region.unwrap().base
    		);
    }
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
