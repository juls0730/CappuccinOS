pub mod acpi;
pub mod fs;
pub mod keyboard;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod pci;
pub mod serial;
pub mod storage;
pub mod video;
