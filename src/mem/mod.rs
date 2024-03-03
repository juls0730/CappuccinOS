pub mod allocator;
pub mod pmm;

use core::alloc::GlobalAlloc;

use limine::{MemmapEntry, NonNullPtr};

use crate::libs::{cell::LazyCell, sync::Mutex};

use self::{allocator::BuddyAllocator, pmm::PhysicalMemoryManager};

static MEMMAP_REQUEST: limine::MemmapRequest = limine::MemmapRequest::new(0);
static HHDM_REQUEST: limine::HhdmRequest = limine::HhdmRequest::new(0);

pub static MEMMAP: LazyCell<Mutex<&mut [NonNullPtr<MemmapEntry>]>> = LazyCell::new(|| {
    let memmap_request = MEMMAP_REQUEST
        .get_response()
        .get_mut()
        .expect("Failed to get Memory map!");

    return Mutex::new(memmap_request.memmap_mut());
});

pub static HHDM_OFFSET: LazyCell<usize> = LazyCell::new(|| {
    let hhdm = HHDM_REQUEST
        .get_response()
        .get()
        .expect("Failed to get Higher Half Direct Map!");

    return hhdm.offset as usize;
});

pub static PHYSICAL_MEMORY_MANAGER: LazyCell<PhysicalMemoryManager> =
    LazyCell::new(PhysicalMemoryManager::new);

pub struct Allocator {
    pub inner: LazyCell<BuddyAllocator>,
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
    inner: LazyCell::new(|| {
        let heap_start = PHYSICAL_MEMORY_MANAGER
            .alloc(HEAP_PAGES)
            .expect("Failed to allocate heap!");

        BuddyAllocator::new_unchecked(heap_start, HEAP_SIZE)
    }),
};

pub fn log_memory_map() {
    let memmap_request = MEMMAP_REQUEST.get_response().get_mut();
    if memmap_request.is_none() {
        panic!("Memory map was None!");
    }

    let memmap = memmap_request.unwrap().memmap();

    crate::log_serial!("====== MEMORY MAP ======\n");
    for entry in memmap.iter() {
        let label = (entry.len as usize).label_bytes();

        crate::log_serial!(
            "[ {:#018X?} ] Type: {:?} Size: {}\n",
            entry.base..entry.base + entry.len,
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
        let bytes = *self;

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

/// # Safety
/// This will produce undefined behavior if dst is not valid for count writes
pub unsafe fn memset32(dst: *mut u32, val: u32, count: usize) {
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        let mut buf = dst;
        unsafe {
            while buf < dst.add(count) {
                core::ptr::write_volatile(buf, val);
                buf = buf.offset(1);
            }
        }
        return;
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        core::arch::asm!(
            "rep stosd",
            inout("ecx") count => _,
            inout("edi") dst => _,
            inout("eax") val => _
        );
    }
}
