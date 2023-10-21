// Physical Memory Manager (pmm)

use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

pub const PAGE_SIZE: usize = 4096;

#[derive(Debug)]
pub struct PhysicalMemoryManager {
    bitmap: AtomicPtr<u8>,
    highest_page_index: AtomicUsize,
    last_used_index: AtomicUsize,
    usable_pages: AtomicUsize,
    used_pages: AtomicUsize,
    reserved_pages: AtomicUsize,
}

impl PhysicalMemoryManager {
    pub fn new() -> Self {
        let pmm = Self {
            bitmap: AtomicPtr::new(core::ptr::null_mut()),
            highest_page_index: AtomicUsize::new(0),
            last_used_index: AtomicUsize::new(0),
            usable_pages: AtomicUsize::new(0),
            used_pages: AtomicUsize::new(0),
            reserved_pages: AtomicUsize::new(0),
        };

        let hhdm_offset = *super::HHDM_OFFSET;

        let mut highest_addr: usize = 0;

        for entry in super::MEMMAP.lock().read().iter() {
            match entry.typ {
                limine::MemoryMapEntryType::Usable => {
                    pmm.usable_pages
                        .fetch_add(entry.len as usize / PAGE_SIZE, Ordering::SeqCst);
                    if highest_addr < (entry.base + entry.len) as usize {
                        highest_addr = (entry.base + entry.len) as usize;
                    }
                }
                _ => {
                    pmm.reserved_pages
                        .fetch_add(entry.len as usize / PAGE_SIZE, Ordering::SeqCst);
                }
            }
        }

        pmm.highest_page_index
            .store(highest_addr / PAGE_SIZE, Ordering::SeqCst);
        let bitmap_size = ((pmm.highest_page_index.load(Ordering::SeqCst) / 8) + PAGE_SIZE - 1)
            & !(PAGE_SIZE - 1);

        for entry in super::MEMMAP.lock().write().iter_mut() {
            if entry.typ != limine::MemoryMapEntryType::Usable {
                continue;
            }

            if entry.len as usize >= bitmap_size {
                let ptr = (entry.base as usize + hhdm_offset) as *mut u8;
                pmm.bitmap.store(ptr, Ordering::SeqCst);

                unsafe {
                    // Set the bit map to non-free
                    core::ptr::write_bytes(ptr, 0xFF, bitmap_size);
                };

                entry.len -= bitmap_size as u64;
                entry.base += bitmap_size as u64;

                break;
            }
        }

        for entry in super::MEMMAP.lock().read().iter() {
            if entry.typ != limine::MemoryMapEntryType::Usable {
                continue;
            }

            let mut i: usize = 0;
            loop {
                if i >= entry.len as usize {
                    break;
                }

                pmm.bitmap_reset((entry.base as usize + i) / PAGE_SIZE);
                i += PAGE_SIZE;
            }
        }

        return pmm;
    }

    fn inner_alloc(&self, pages: usize, limit: usize) -> *mut u8 {
        let mut p: usize = 0;

        while self.last_used_index.load(Ordering::SeqCst) < limit {
            if self.bitmap_test(self.last_used_index.fetch_add(1, Ordering::SeqCst)) != true {
                p += 1;
                if p == pages {
                    let page = self.last_used_index.load(Ordering::SeqCst) - pages;
                    for i in page..self.last_used_index.load(Ordering::SeqCst) {
                        self.bitmap_set(i);
                    }
                    return (page * PAGE_SIZE) as *mut u8;
                }
            } else {
                p = 0;
            }
        }

        return core::ptr::null_mut();
    }

    pub fn alloc_nozero(&self, pages: usize) -> Result<*mut u8, ()> {
        let last = self.last_used_index.load(Ordering::SeqCst);
        let mut ret = self.inner_alloc(pages, self.highest_page_index.load(Ordering::SeqCst));

        if ret.is_null() {
            self.last_used_index.store(0, Ordering::SeqCst);
            ret = self.inner_alloc(pages, last);

            // If ret is still null, we have ran out of memory, panic
            if ret.is_null() {
                return Err(());
            }
        }

        self.used_pages.fetch_add(pages, Ordering::SeqCst);

        return Ok(ret);
    }

    pub fn alloc(&self, pages: usize) -> Result<*mut u8, ()> {
        let ret = self.alloc_nozero(pages)?;
        unsafe {
            core::ptr::write_bytes(ret, 0x00, pages * PAGE_SIZE);
        };

        return Ok(ret);
    }

    pub fn dealloc(&self, addr: *mut u8, pages: usize) {
        let page = (addr as *mut u64).addr() / PAGE_SIZE;

        let mut i: usize = 0;
        loop {
            if i >= page + pages {
                break;
            }

            self.bitmap_reset(i);
            i += 1;
        }

        self.used_pages.fetch_sub(pages, Ordering::SeqCst);
    }

    #[inline(always)]
    fn bitmap_test(&self, bit: usize) -> bool {
        unsafe {
            let byte_index = bit / 8;
            let bit_index = bit % 8;
            // (*self.bitmap.lock().write().add(byte_index)) & (1 << bit_index) != 0
            return (*self.bitmap.load(Ordering::SeqCst).add(byte_index)) & (1 << bit_index) != 0;
        }
    }

    #[inline(always)]
    fn bitmap_set(&self, bit: usize) {
        unsafe {
            let byte_index = bit / 8;
            let bit_index = bit % 8;
            (*self.bitmap.load(Ordering::SeqCst).add(byte_index)) |= 1 << bit_index;
        }
    }

    #[inline(always)]
    fn bitmap_reset(&self, bit: usize) {
        unsafe {
            let byte_index = bit / 8;
            let bit_index = bit % 8;
            (*self.bitmap.load(Ordering::SeqCst).add(byte_index)) &= !(1 << bit_index);
        }
    }

    pub fn total_memory(&self) -> usize {
        return self.usable_pages.load(Ordering::SeqCst) * 4096;
    }

    pub fn usable_memory(&self) -> usize {
        return (self.usable_pages.load(Ordering::SeqCst) * 4096)
            - (self.used_pages.load(Ordering::SeqCst) * 4096);
    }

    pub fn used_memory(&self) -> usize {
        return self.used_pages.load(Ordering::SeqCst) * 4096;
    }
}
