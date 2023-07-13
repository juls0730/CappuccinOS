pub use self::imp::interrupts;

#[cfg(target_arch = "x86_64")]
#[path = "x86_64"]
mod imp {
    pub mod interrupts;
}

mod x86_common;
