pub mod allocator;
pub mod pmm;

use core::alloc::GlobalAlloc;

use limine::{MemmapEntry, NonNullPtr};

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
