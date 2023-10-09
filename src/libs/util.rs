#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub unsafe fn memset32(dst: *mut u32, val: u32, count: usize) {
    if cfg!(not(any(target_arch = "x86", target_arch = "x86_64"))) {
        let mut buf = dst;
        unsafe {
            while buf < dst.offset(count as isize) {
                core::ptr::write_volatile(buf, val);
                buf = buf.offset(1);
            }
        }
        return;
    }

    core::arch::asm!(
        "rep stosd",
        inout("ecx") count => _,
        inout("edi") dst => _,
        inout("eax") val => _
    );
}

pub fn hcf() -> ! {
    loop {
        unsafe {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            core::arch::asm!("hlt");

            #[cfg(target_arch = "aarch64")]
            core::arch::asm!("wfi");
        }
    }
}
