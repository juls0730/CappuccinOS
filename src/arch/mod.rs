pub use self::imp::*;
pub use self::x86_common::*;

mod x86_common;
#[cfg(target_arch = "x86_64")]
#[path = "x86_64"]
mod imp {
    pub mod interrupts;
}
