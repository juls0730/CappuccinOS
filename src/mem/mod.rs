pub mod allocator;
pub mod pmm;

use core::alloc::GlobalAlloc;

use alloc::string::{String, ToString};
use limine::{MemmapEntry, MemoryMapEntryType, NonNullPtr};

use crate::{
    libs::{lazy::Lazy, mutex::Mutex},
    usr::tty::CONSOLE,
};

use self::{allocator::BuddyAllocator, pmm::PhysicalMemoryManager};

static MEMMAP_REQUEST: limine::MemmapRequest = limine::MemmapRequest::new(0);
static HHDM_REQUEST: limine::HhdmRequest = limine::HhdmRequest::new(0);

pub static MEMMAP: Lazy<Mutex<&mut [NonNullPtr<MemmapEntry>]>> = Lazy::new(|| {
    let memmap_request = MEMMAP_REQUEST
        .get_response()
        .get_mut()
        .expect("Failed to get Memory map!");

    return Mutex::new(memmap_request.memmap_mut());
});

pub static HHDM_OFFSET: Lazy<usize> = Lazy::new(|| {
    let hhdm = HHDM_REQUEST
        .get_response()
        .get()
        .expect("Failed to get Higher Half Direct Map!");

    return hhdm.offset as usize;
});

// fn stitch_memory_map(memmap: &mut [NonNullPtr<MemmapEntry>]) -> &mut [NonNullPtr<MemmapEntry>] {
//     let mut null_index_ptr = 0;
//
//     crate::println!("====== MEMORY MAP ======");
//     for entry in memmap.iter() {
//         crate::println!(
//             "[ {:#018X?} ] Type: {:?} Size: {:?}",
//             entry.base..entry.base + entry.len,
//             entry.typ,
//             entry.len as usize
//         )
//     }
//
//     for i in 1..memmap.len() {
//         let region = &memmap[i];
//
//         #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
//         if region.typ == limine::MemoryMapEntryType::Framebuffer {
//             crate::arch::set_mtrr(
//                 region.base,
//                 region.len,
//                 crate::arch::MTRRMode::WriteCombining,
//             );
//         }
//
//         if !memory_section_is_usable(&memmap[i]) {
//             memmap[null_index_ptr].len = memmap[i].len;
//             memmap[null_index_ptr].base = memmap[i].base;
//             memmap[null_index_ptr].typ = memmap[i].typ;
//
//             null_index_ptr += 1;
//             continue;
//         }
//
//         if null_index_ptr > 0 && memory_section_is_usable(&memmap[null_index_ptr - 1]) {
//             memmap[null_index_ptr - 1].len += memmap[i].len;
//
//             continue;
//         }
//
//         if memory_section_is_usable(&memmap[i - 1]) {
//             memmap[i - 1].len += memmap[i].len;
//
//             memmap[null_index_ptr].len = memmap[i - 1].len;
//             memmap[null_index_ptr].base = memmap[i - 1].base;
//             memmap[null_index_ptr].typ = memmap[i - 1].typ;
//
//             null_index_ptr += 1;
//             continue;
//         }
//     }
//
//     return &mut memmap[0..null_index_ptr];
// }

// pub static LARGEST_MEMORY_REGIONS: Lazy<(
//     &NonNullPtr<MemmapEntry>,
//     Option<&NonNullPtr<MemmapEntry>>,
// )> = Lazy::new(|| {
//     let memmap = MEMMAP.lock().read();

//     let min_heap_size = 0x0008_0000;
//     let mut largest_region: Option<&NonNullPtr<MemmapEntry>> = None;
//     let mut second_largest_region: Option<&NonNullPtr<MemmapEntry>> = None;
//     let mut framebuffer_region: Option<&NonNullPtr<MemmapEntry>> = None;

//     for region in memmap.iter() {
//         if region.typ == limine::MemoryMapEntryType::Framebuffer {
//             framebuffer_region = Some(region);
//             continue;
//         }

//         if !memory_section_is_usable(region) {
//             continue;
//         }

//         if largest_region.is_none() || region.len > largest_region.unwrap().len {
//             second_largest_region = largest_region;
//             largest_region = Some(region);
//         } else if second_largest_region.is_none() || region.len > second_largest_region.unwrap().len
//         {
//             second_largest_region = Some(region);
//         }
//     }

//     if largest_region.is_none() {
//         panic!("Suitable memory regions not found!");
//     }

//     let largest_region = largest_region.unwrap();

//     if framebuffer_region.is_none() || second_largest_region.is_none() {
//         return (largest_region, None);
//     }

//     let framebuffer_size = framebuffer_region.unwrap().len;

//     if second_largest_region.unwrap().len >= framebuffer_size {
//         return (largest_region, second_largest_region);
//     }

//     let shrunk_heap = largest_region.len - framebuffer_size;

//     if shrunk_heap < min_heap_size as u64 {
//         return (largest_region, None);
//     }

//     unsafe {
//         (*second_largest_region.unwrap().as_ptr()).base = largest_region.base;

//         (*largest_region.as_ptr()).len = shrunk_heap;
//         (*second_largest_region.unwrap().as_ptr()).base += shrunk_heap;
//     }

//     return (largest_region, second_largest_region);
// });

pub static PHYSICAL_MEMORY_MANAGER: Lazy<PhysicalMemoryManager> =
    Lazy::new(|| PhysicalMemoryManager::new());

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

const HEAP_PAGES: usize = 4096;
const HEAP_SIZE: usize = HEAP_PAGES * 1024;

#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator {
    inner: Lazy::new(|| {
        let heap_start = PHYSICAL_MEMORY_MANAGER
            .alloc(HEAP_PAGES)
            .expect("Failed to allocate heap!");

        BuddyAllocator::new_unchecked(heap_start, HEAP_SIZE)
    }),
};

#[inline]
fn memory_section_is_usable(entry: &MemmapEntry) -> bool {
    return entry.typ == MemoryMapEntryType::Usable;
}

pub fn log_info() {
    crate::log_info!(
        "Initialized heap with {} of memory at {:#X}",
        HEAP_SIZE.label_bytes(),
        ALLOCATOR
            .inner
            .heap_start
            .load(core::sync::atomic::Ordering::SeqCst) as usize
    );

    if CONSOLE.get_features().doubled_buffered {
        let row_size = CONSOLE.second_buffer.lock().read().unwrap().pitch
            / (CONSOLE.second_buffer.lock().read().unwrap().bpp / 8);

        let screen_size = row_size * CONSOLE.second_buffer.lock().read().unwrap().height;
        crate::log_info!(
            "Initialized framebuffer mirroring with {} at {:#X}",
            (screen_size * 4).label_bytes(),
            CONSOLE.second_buffer.lock().read().unwrap().pointer as usize
        );
    }

    log_memory_map();
}

pub fn log_memory_map() {
    let memmap_request = MEMMAP_REQUEST.get_response().get_mut();
    if memmap_request.is_none() {
        panic!("Memory map was None!");
    }

    let memmap = memmap_request.unwrap().memmap();

    crate::println!("====== MEMORY MAP ======");
    for entry in memmap.iter() {
        let label = (entry.len as usize).label_bytes();

        crate::println!(
            "[ {:#018X?} ] Type: \033[{};m{:?}\033[0;m Size: {}",
            entry.base..entry.base + entry.len,
            match entry.typ {
                limine::MemoryMapEntryType::Usable => 32,
                _ => 31,
            },
            entry.typ,
            label
        )
    }
}

pub struct Label {
    size: usize,
    text_label: &'static str,
}

impl core::fmt::Display for Label {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        return write!(f, "{}{}", self.size, self.text_label);
    }
}

// Hacky solution to avoid allocation, but keep the names
static BYTE_LABELS: (&str, &str, &str, &str) = ("GiB", "MiB", "KiB", "Bytes");

pub trait LabelBytes {
    fn label_bytes(&self) -> Label;
}

impl LabelBytes for usize {
    fn label_bytes(&self) -> Label {
        let bytes = self.clone();

        if bytes >> 30 > 0 {
            return Label {
                size: bytes >> 30,
                text_label: BYTE_LABELS.0,
            };
        } else if bytes >> 20 > 0 {
            return Label {
                size: bytes >> 20,
                text_label: BYTE_LABELS.1,
            };
        } else if bytes >> 10 > 0 {
            return Label {
                size: bytes >> 10,
                text_label: BYTE_LABELS.2,
            };
        } else {
            return Label {
                size: bytes,
                text_label: BYTE_LABELS.3,
            };
        }
    }
}
