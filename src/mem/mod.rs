pub mod allocator;

use core::alloc::GlobalAlloc;

use limine::{MemmapEntry, MemoryMapEntryType, NonNullPtr};

use crate::libs::lazy::Lazy;

use self::allocator::BuddyAllocator;

static MEMMAP_REQUEST: limine::MemmapRequest = limine::MemmapRequest::new(0);

// fn stitch_memory_map(memmap: &mut [NonNullPtr<MemmapEntry>]) -> &mut [NonNullPtr<MemmapEntry>] {
//     let mut null_index_ptr = 0;

//     crate::println!("====== MEMORY MAP ======");
//     for entry in memmap.iter() {
//         crate::println!(
//             "[ {:#018X?} ] Type: {:?} Size: {:?}",
//             entry.base..entry.base + entry.len,
//             entry.typ,
//             entry.len as usize
//         )
//     }

//     for i in 1..memmap.len() {
//         let region = &memmap[i];

//         #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
//         if region.typ == limine::MemoryMapEntryType::Framebuffer {
//             crate::arch::set_mtrr(
//                 region.base,
//                 region.len,
//                 crate::arch::MTRRMode::WriteCombining,
//             );
//         }

//         if !memory_section_is_usable(&memmap[i]) {
//             memmap[null_index_ptr].len = memmap[i].len;
//             memmap[null_index_ptr].base = memmap[i].base;
//             memmap[null_index_ptr].typ = memmap[i].typ;

//             null_index_ptr += 1;
//             continue;
//         }

//         if null_index_ptr > 0 && memory_section_is_usable(&memmap[null_index_ptr - 1]) {
//             memmap[null_index_ptr - 1].len += memmap[i].len;

//             continue;
//         }

//         if memory_section_is_usable(&memmap[i - 1]) {
//             memmap[i - 1].len += memmap[i].len;

//             memmap[null_index_ptr].len = memmap[i - 1].len;
//             memmap[null_index_ptr].base = memmap[i - 1].base;
//             memmap[null_index_ptr].typ = memmap[i - 1].typ;

//             null_index_ptr += 1;
//             continue;
//         }
//     }

//     return &mut memmap[0..null_index_ptr];
// }

pub static LARGEST_MEMORY_REGIONS: Lazy<(
    &NonNullPtr<MemmapEntry>,
    Option<&NonNullPtr<MemmapEntry>>,
)> = Lazy::new(|| {
    let memmap_request = MEMMAP_REQUEST.get_response().get_mut();
    if memmap_request.is_none() {
        panic!("Memory map was None!");
    }

    let memmap = memmap_request.unwrap().memmap();

    let min_heap_size = 0x0008_0000;
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

    if largest_region.is_none() {
        panic!("Suitable memory regions not found!");
    }

    let largest_region = largest_region.unwrap();

    if framebuffer_region.is_none() || second_largest_region.is_none() {
        return (largest_region, None);
    }

    let framebuffer_size = framebuffer_region.unwrap().len;

    if second_largest_region.unwrap().len >= framebuffer_size {
        return (largest_region, second_largest_region);
    }

    let shrunk_heap = largest_region.len - framebuffer_size;

    if shrunk_heap < min_heap_size as u64 {
        return (largest_region, None);
    }

    unsafe {
        (*second_largest_region.unwrap().as_ptr()).base = largest_region.base;

        (*largest_region.as_ptr()).len = shrunk_heap;
        (*second_largest_region.unwrap().as_ptr()).base += shrunk_heap;
    }

    return (largest_region, second_largest_region);
});

pub struct Allocator {
    pub inner: Lazy<BuddyAllocator>,
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.inner.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        self.inner.dealloc(ptr, layout)
    }
}

#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator {
    inner: Lazy::new(|| {
        BuddyAllocator::new_unchecked(
            LARGEST_MEMORY_REGIONS.0.base as *mut u8,
            LARGEST_MEMORY_REGIONS.0.len as usize,
        )
    }),
};

#[inline]
fn memory_section_is_usable(entry: &MemmapEntry) -> bool {
    return entry.typ == MemoryMapEntryType::Usable;
}

pub fn log_info() {
    let (largest_region, second_largest_region) = *LARGEST_MEMORY_REGIONS;

    crate::log_info!(
        "Using largest section with: {} bytes of memory for heap at {:#X}",
        largest_region.len,
        largest_region.base
    );

    if second_largest_region.is_some() {
        crate::log_info!(
            "Using second largest section with: {} bytes of memory for framebuffer mirroring at {:#X}",
            second_largest_region.unwrap().len,
            second_largest_region.unwrap().base
        );
    }

    log_memory_map();
}

fn log_memory_map() {
    let memmap_request = MEMMAP_REQUEST.get_response().get_mut();
    if memmap_request.is_none() {
        panic!("Memory map was None!");
    }

    let memmap = memmap_request.unwrap().memmap();

    crate::println!("====== MEMORY MAP ======");
    for entry in memmap.iter() {
        let label = label_units(entry.len as usize);

        crate::println!(
            "[ {:#018X?} ] Type: \033[{};m{:?}\033[0;m Size: {} {}",
            entry.base..entry.base + entry.len,
            match entry.typ {
                limine::MemoryMapEntryType::Usable => 32,
                _ => 31,
            },
            entry.typ,
            label.0,
            label.1
        )
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
