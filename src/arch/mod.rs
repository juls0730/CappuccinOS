#[cfg(any(target_arch = "x86_64"))]
pub use self::imp::*;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use self::x86_common::*;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86_common;
#[cfg(target_arch = "x86_64")]
#[path = "x86_64"]
mod imp {
    pub mod interrupts;
}
