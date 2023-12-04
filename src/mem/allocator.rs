// Original code from: https://github.com/DrChat/buddyalloc/blob/master/src/heap.rs
// But I made it ~~much worse~~ *better* by making it GlobalAlloc compatible
// By using A custom Mutex implementation (which also sucks) and dereferencing all the pointers,
// I was able to remove all the mut's In the original code.

// TODO: Replace this with a slab allocator that can take advantage of the page frame allocator

use core::alloc::{GlobalAlloc, Layout};
use core::cmp::{max, min};
use core::ptr;
use core::sync::atomic::Ordering::SeqCst;
use core::sync::atomic::{AtomicPtr, AtomicU8, AtomicUsize};

use crate::libs::mutex::Mutex;

const fn log2(num: usize) -> u8 {
    let mut temp = num;
    let mut result = 0;

    temp >>= 1;

    while temp != 0 {
        result += 1;
        temp >>= 1;
    }

    return result;
}

const MIN_HEAP_ALIGN: usize = 4096;
const HEAP_BLOCKS: usize = 16;

pub struct FreeBlock {
    next: *mut FreeBlock,
}

impl FreeBlock {
    #[inline]
    const fn new(next: *mut FreeBlock) -> Self {
        Self { next }
    }
}

pub struct BuddyAllocator {
    pub heap_start: AtomicPtr<u8>,
    heap_size: AtomicUsize,
    free_lists: Mutex<[*mut FreeBlock; HEAP_BLOCKS]>,
    min_block_size: AtomicUsize,
    min_block_size_log2: AtomicU8,
}

impl BuddyAllocator {
    pub const fn new_unchecked(heap_start: *mut u8, heap_size: usize) -> Self {
        let min_block_size_raw = heap_size >> (HEAP_BLOCKS - 1);
        let min_block_size = AtomicUsize::new(min_block_size_raw);
        let mut free_lists_buf: [*mut FreeBlock; HEAP_BLOCKS] = [ptr::null_mut(); HEAP_BLOCKS];

        free_lists_buf[HEAP_BLOCKS - 1] = heap_start as *mut FreeBlock;

        let free_lists: Mutex<[*mut FreeBlock; HEAP_BLOCKS]> = Mutex::new(free_lists_buf);

        let heap_start = AtomicPtr::new(heap_start);
        let heap_size = AtomicUsize::new(heap_size);

        Self {
            heap_start,
            heap_size,
            free_lists,
            min_block_size,
            min_block_size_log2: AtomicU8::new(log2(min_block_size_raw)),
        }
    }

    fn allocation_size(&self, mut size: usize, align: usize) -> Option<usize> {
        if !align.is_power_of_two() {
            return None;
        }

        if align > MIN_HEAP_ALIGN {
            return None;
        }

        if align > size {
            size = align;
        }

        size = max(size, self.min_block_size.load(SeqCst));

        size = size.next_power_of_two();

        if size > self.heap_size.load(SeqCst) {
            return None;
        }

        return Some(size);
    }

    fn allocation_order(&self, size: usize, align: usize) -> Option<usize> {
        return self
            .allocation_size(size, align)
            .map(|s| (log2(s) - self.min_block_size_log2.load(SeqCst)) as usize);
    }

    #[inline]
    fn order_size(&self, order: usize) -> usize {
        return 1 << (self.min_block_size_log2.load(SeqCst) as usize + order);
    }

    fn free_list_pop(&self, order: usize) -> Option<*mut u8> {
        let candidate = (*self.free_lists.lock().read())[order];

        if candidate.is_null() {
            return None;
        }

        if order != self.free_lists.lock().read().len() - 1 {
            (*self.free_lists.lock().write())[order] = unsafe { (*candidate).next };
        } else {
            (*self.free_lists.lock().write())[order] = ptr::null_mut();
        }

        return Some(candidate as *mut u8);
    }

    fn free_list_insert(&self, order: usize, block: *mut u8) {
        let free_block_ptr = block as *mut FreeBlock;

        unsafe { *free_block_ptr = FreeBlock::new((*self.free_lists.lock().read())[order]) };

        (*self.free_lists.lock().write())[order] = free_block_ptr;
    }

    fn free_list_remove(&self, order: usize, block: *mut u8) -> bool {
        let block_ptr = block as *mut FreeBlock;

        let mut checking: &mut *mut FreeBlock = &mut (*self.free_lists.lock().write())[order];

        unsafe {
            while !(*checking).is_null() {
                if *checking == block_ptr {
                    *checking = (*(*checking)).next;
                    return true;
                }

                checking = &mut ((*(*checking)).next);
            }
        }
        return false;
    }

    fn split_free_block(&self, block: *mut u8, mut order: usize, order_needed: usize) {
        let mut size_to_split = self.order_size(order);

        while order > order_needed {
            size_to_split >>= 1;
            order -= 1;

            let split = unsafe { block.add(size_to_split) };
            self.free_list_insert(order, split);
        }
    }

    fn buddy(&self, order: usize, block: *mut u8) -> Option<*mut u8> {
        assert!(block >= self.heap_start.load(SeqCst));

        let relative = unsafe { block.offset_from(self.heap_start.load(SeqCst)) } as usize;
        let size = self.order_size(order);
        if size >= self.heap_size.load(SeqCst) {
            return None;
        } else {
            return Some(unsafe { self.heap_start.load(SeqCst).add(relative ^ size) });
        }
    }

    pub fn get_total_mem(&self) -> usize {
        return self.heap_size.load(SeqCst);
    }

    pub fn get_free_mem(&self) -> usize {
        let mut free_mem = 0;

        unsafe {
            for order in 0..self.free_lists.lock().read().len() {
                let mut block = (*self.free_lists.lock().write())[order];

                while !block.is_null() {
                    free_mem += self.order_size(order);
                    block = (*block).next;
                }
            }
        }

        return free_mem;
    }

    pub fn get_used_mem(&self) -> usize {
        return self.get_total_mem() - self.get_free_mem();
    }
}

unsafe impl GlobalAlloc for BuddyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Some(order_needed) = self.allocation_order(layout.size(), layout.align()) {
            for order in order_needed..self.free_lists.lock().read().len() {
                if let Some(block) = self.free_list_pop(order) {
                    if order > order_needed {
                        self.split_free_block(block, order, order_needed);
                    }

                    return block;
                }
            }
        }

        return ptr::null_mut();
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let initial_order = self
            .allocation_order(layout.size(), layout.align())
            .expect("Tried to dispose of invalid block");

        let mut block = ptr;
        for order in initial_order..self.free_lists.lock().read().len() {
            if let Some(buddy) = self.buddy(order, block) {
                if self.free_list_remove(order, block) {
                    block = min(block, buddy);
                    continue;
                }
            }

            self.free_list_insert(order, block);
            return;
        }
    }
}
