use core::{cell::UnsafeCell, ops::Deref, sync::atomic::AtomicU8};

pub struct OnceCell<T> {
    state: AtomicU8,
    data: UnsafeCell<Option<T>>,
}

unsafe impl<T> Sync for OnceCell<T> {}

#[repr(u8)]
enum State {
    Uninitialized = 0,
    Initializing,
    Initialized,
}

impl From<u8> for State {
    fn from(value: u8) -> Self {
        match value {
            0 => State::Uninitialized,
            1 => State::Initializing,
            2 => State::Initialized,
            _ => panic!("Invalid state value"),
        }
    }
}

impl<T> OnceCell<T> {
    pub const fn new() -> Self {
        return OnceCell {
            state: AtomicU8::new(State::Uninitialized as u8),
            data: UnsafeCell::new(None),
        };
    }

    pub fn set(&self, new_data: T) {
        let current_state = self.state.load(core::sync::atomic::Ordering::SeqCst);

        match State::from(current_state) {
            State::Uninitialized => {
                self.state.store(
                    State::Initializing as u8,
                    core::sync::atomic::Ordering::SeqCst,
                );

                unsafe { *self.data.get() = Some(new_data) };

                self.state.store(
                    State::Initialized as u8,
                    core::sync::atomic::Ordering::SeqCst,
                );
            }
            _ => panic!("Tried to initialize data that is alread initialized"),
        }
    }
}

impl<T> Deref for OnceCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        if self.state.load(core::sync::atomic::Ordering::SeqCst) == State::Initialized as u8 {
            if let Some(value) = unsafe { &*self.data.get() } {
                return value;
            }
        }

        panic!("Attempted to access uninitialized data!")
    }
}
