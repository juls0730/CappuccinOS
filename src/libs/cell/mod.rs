use core::cell::UnsafeCell;

mod lazy;
mod once;

pub use lazy::LazyCell;
pub use once::OnceCell;

pub struct Cell<T: ?Sized> {
    value: UnsafeCell<T>,
}

impl<T> Cell<T> {
    pub const fn new(value: T) -> Cell<T> {
        return Self {
            value: UnsafeCell::new(value),
        };
    }

    pub fn get(&self) -> &T {
        return unsafe { &*self.value.get() };
    }

    pub fn set(&self, new_value: T) {
        unsafe { *self.value.get() = new_value };
    }
}
