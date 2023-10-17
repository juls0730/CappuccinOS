use core::{
    cell::UnsafeCell,
    ops::Deref,
    sync::atomic::{AtomicBool, Ordering},
};

pub struct Lazy<T, F = fn() -> T> {
    value: UnsafeCell<Option<T>>,
    init_func: Option<F>,
    initialized: AtomicBool,
}

impl<T, F: Fn() -> T> Lazy<T, F> {
    pub const fn new(init_func: F) -> Self {
        Lazy {
            value: UnsafeCell::new(None),
            init_func: Some(init_func),
            initialized: AtomicBool::new(false),
        }
    }
}

impl<T, F: Fn() -> T> Deref for Lazy<T, F> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        if !self.initialized.load(Ordering::Acquire) {
            if let Some(init_func) = &self.init_func {
                let value = init_func();
                unsafe {
                    *(self.value.get()) = Some(value);
                }
                self.initialized.store(true, Ordering::Release);
            }
        }

        unsafe {
            (*self.value.get())
                .as_ref()
                .expect("Lazy value is not initialized!")
        }
    }
}

unsafe impl<T, F: Fn() -> T + Send> Sync for Lazy<T, F> {}
