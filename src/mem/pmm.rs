// Physical Memory Manager (pmm)

const PAGE_SIZE: usize = 4096;

pub struct PhysicalMemoryManager {
    bitmap: *mut u8,
    highest_page_index: usize,
    last_used_index: usize,
    usable_pages: usize,
    used_pages: usize,
    reserved_pages: usize,
}

impl PhysicalMemoryManager {
    pub fn new() -> Self {
        let mut pmm = Self {
            bitmap: core::ptr::null_mut(),
            highest_page_index: 0,
            last_used_index: 0,
            usable_pages: 0,
            used_pages: 0,
            reserved_pages: 0,
        };

        let hhdm_offset = *super::HHDM_OFFSET;

        let mut highest_addr: usize = 0;

        for entry in super::MEMMAP.lock().read().iter() {
            match entry.typ {
                limine::MemoryMapEntryType::Usable => {
                    pmm.usable_pages += entry.len as usize / PAGE_SIZE;
                    if highest_addr < (entry.base + entry.len) as usize {
                        highest_addr = (entry.base + entry.len) as usize;
                    }
                }
                _ => pmm.reserved_pages += entry.len as usize / PAGE_SIZE,
            }
        }

        pmm.highest_page_index = highest_addr / PAGE_SIZE;
        let bitmap_size = ((pmm.highest_page_index / 8) + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);

        for entry in super::MEMMAP.lock().write().iter_mut() {
            if entry.typ != limine::MemoryMapEntryType::Usable {
                continue;
            }

            if entry.len as usize >= bitmap_size {
                pmm.bitmap = (entry.base as usize + hhdm_offset) as *mut u8;

                unsafe {
                    // Set the bit map to non-free
                    core::ptr::write_bytes(pmm.bitmap, 0xFF, bitmap_size);
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

                // reset bitmap
                let bit = (entry.base as usize + i) / PAGE_SIZE;
                unsafe {
                    *(pmm.bitmap.add(bit / 8)) &= !(1 << (bit % 8));
                };
                i += PAGE_SIZE;
            }
        }

        return pmm;
    }

    fn inner_alloc(&mut self, pages: usize, limit: usize) -> *mut u8 {
        let mut p: usize = 0;

        while self.last_used_index < limit {
            self.last_used_index += 1;
            let bit = self.last_used_index;
            let bitmap_u8 = unsafe { *self.bitmap.add(bit / 8) };

            if (bitmap_u8 & (1 << (bit % 8))) == 0 {
                p += 1;
                if p == pages {
                    let page = self.last_used_index - pages;
                    for i in 0..self.last_used_index {
                        unsafe {
                            *self.bitmap.add(i / 8) |= 1 << (bit % 8);
                        };
                    }
                    return (page * PAGE_SIZE) as *mut u8;
                }
            } else {
                p = 0;
            }
        }

        return core::ptr::null_mut();
    }

    pub fn alloc_nozero(&mut self, pages: usize) -> *mut u8 {
        let last = self.last_used_index;
        let mut ret = self.inner_alloc(pages, self.highest_page_index);

        if ret.is_null() {
            self.last_used_index = 0;
            ret = self.inner_alloc(pages, last);

            // If ret is still null, we have ran out of memory, panic
            if ret.is_null() {
                panic!("Out of memory!");
            }
        }

        self.used_pages += pages;

        return ret;
    }

    pub fn alloc(&mut self, pages: usize) -> *mut u8 {
        let ret = self.alloc_nozero(pages);
        unsafe {
            core::ptr::write_bytes(ret, 0x00, pages * PAGE_SIZE);
        };

        return ret;
    }

    pub fn dealloc(&mut self, addr: *mut u8, pages: usize) {
        let page = (addr as *mut u64).addr() / PAGE_SIZE;

        let mut i: usize = 0;
        loop {
            if i >= page + pages {
                break;
            }

            let bit = i;
            unsafe {
                *(self.bitmap.add(bit / 8)) &= !(1 << (bit % 8));
            };
            i += 1;
        }

        self.used_pages -= pages;
    }

    pub fn usable_memory(&self) -> usize {
        return self.usable_pages * 4096;
    }

    pub fn used_memory(&self) -> usize {
        return self.used_pages * 4096;
    }
}
