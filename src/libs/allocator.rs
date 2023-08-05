// Original code from: https://github.com/DrChat/buddyalloc/blob/master/src/heap.rs
// But I made it ~~much worse~~ *better* by making it GlobalAlloc compatible
// By using UnsafeCell and dereferencing all the pointers, I was able to remove all the mut's
// In the original code.

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::cmp::{max, min};
use core::mem::size_of;
use core::ptr::{self, NonNull};

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
const HEAP_START: usize = 0x00FF_0000;
const HEAP_SIZE: usize = 0x0008_0000;
const HEAP_BLOCKS: usize = 16;

struct FreeBlock {
    next: *mut FreeBlock,
}

impl FreeBlock {
    const fn new(next: *mut FreeBlock) -> Self {
        Self { next }
    }
}

pub struct BuddyAllocator {
    heap_start: *mut u8,
    heap_size: usize,
    free_lists: UnsafeCell<[*mut FreeBlock; HEAP_BLOCKS]>,
    min_block_size: usize,
    min_block_size_log2: u8,
}

unsafe impl Sync for BuddyAllocator {}

impl BuddyAllocator {
    pub const fn new_unchecked(heap_start: *mut u8, heap_size: usize) -> Self {
        let min_block_size = heap_size >> (HEAP_BLOCKS - 1);
        let mut free_lists: UnsafeCell<[*mut FreeBlock; HEAP_BLOCKS]> =
            UnsafeCell::new([ptr::null_mut(); HEAP_BLOCKS]);

        (*free_lists.get_mut())[HEAP_BLOCKS - 1] = heap_start as *mut FreeBlock;

        Self {
            heap_start,
            heap_size,
            free_lists,
            min_block_size,
            min_block_size_log2: log2(min_block_size),
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

        size = max(size, self.min_block_size);

        size = size.next_power_of_two();

        if size > self.heap_size {
            return None;
        }

        return Some(size);
    }

    fn allocation_order(&self, size: usize, align: usize) -> Option<usize> {
        return self
            .allocation_size(size, align)
            .map(|s| (log2(s) - self.min_block_size_log2) as usize);
    }

    const fn order_size(&self, order: usize) -> usize {
        return 1 << (self.min_block_size_log2 as usize + order);
    }

    fn free_list_pop(&self, order: usize) -> Option<*mut u8> {
        let candidate = unsafe { (*self.free_lists.get())[order] };

        if candidate.is_null() {
            return None;
        }

        if order != unsafe { (*self.free_lists.get()).len() } - 1 {
            unsafe { (*self.free_lists.get())[order] = (*candidate).next };
        } else {
            unsafe { (*self.free_lists.get())[order] = ptr::null_mut() };
        }

        return Some(candidate as *mut u8);
    }

    unsafe fn free_list_insert(&self, order: usize, block: *mut u8) {
        let free_block_ptr = block as *mut FreeBlock;
        *free_block_ptr = FreeBlock::new(unsafe { (*self.free_lists.get())[order] });
        unsafe { (*self.free_lists.get())[order] = free_block_ptr };
    }

    fn free_list_remove(&self, order: usize, block: *mut u8) -> bool {
        let block_ptr = block as *mut FreeBlock;

        let mut checking: &mut *mut FreeBlock = unsafe { &mut (*self.free_lists.get())[order] };

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

    unsafe fn split_free_block(&self, block: *mut u8, mut order: usize, order_needed: usize) {
        let mut size_to_split = self.order_size(order);

        while order > order_needed {
            size_to_split >>= 1;
            order -= 1;

            let split = block.add(size_to_split);
            self.free_list_insert(order, split);
        }
    }

    fn buddy(&self, order: usize, block: *mut u8) -> Option<*mut u8> {
        assert!(block >= self.heap_start);

        let relative = unsafe { block.offset_from(self.heap_start) } as usize;
        let size = self.order_size(order);
        if size >= self.heap_size {
            return None;
        } else {
            return Some(unsafe { self.heap_start.add(relative ^ size) });
        }
    }
}

unsafe impl GlobalAlloc for BuddyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match self.allocation_order(layout.size(), layout.align()) {
            Some(order_needed) => {
                for order in order_needed..unsafe { (*self.free_lists.get()).len() } {
                    if let Some(block) = self.free_list_pop(order) {
                        if order > order_needed {
                            unsafe { self.split_free_block(block, order, order_needed) };
                        }

                        return block;
                    }
                }

                return ptr::null_mut();
            }
            None => {
                return ptr::null_mut();
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let initial_order = self
            .allocation_order(layout.size(), layout.align())
            .expect("Tried to dispose of invalid block");

        let mut block = ptr;
        for order in initial_order..unsafe { (*self.free_lists.get()).len() } {
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

#[global_allocator]
pub static ALLOCATOR: BuddyAllocator =
    BuddyAllocator::new_unchecked(HEAP_START as *mut u8, HEAP_SIZE);
