use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

pub struct Mutex<T: ?Sized> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    #[inline]
    pub const fn new(data: T) -> Self {
        return Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        };
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        // if self.locked.load(Ordering::Acquire) == true {
        //     unsafe { core::arch::asm!("out dx, al", in("dx") 0x3f8, in("al") 'S' as u8) };
        // }
        while self.locked.swap(true, Ordering::Acquire) {
            // spin lock
        }
        return MutexGuard { mutex: self };
    }
}

impl<T> core::fmt::Debug for Mutex<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let locked = self.locked.load(Ordering::SeqCst);
        write!(f, "Mutex: {{ data: ")?;

        if locked {
            write!(f, "<locked> }}")
        } else {
            write!(f, "{:?} }}", self.data)
        }
    }
}

pub struct MutexGuard<'a, T: ?Sized> {
    mutex: &'a Mutex<T>,
}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // unsafe { core::arch::asm!("out dx, al", in("dx") 0x3f8, in("al") 'D' as u8) };

        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        // unsafe { core::arch::asm!("out dx, al", in("dx") 0x3f8, in("al") 'M' as u8) };
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Release);
    }
}
