pub unsafe fn memset32(dst: *mut u32, val: u32, count: usize) -> *mut u32 {
    let mut buf = dst;

    while buf < dst.offset(count as isize) {
        core::ptr::write_volatile(buf, val);
        buf = buf.offset(1);
    }

    return dst;
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
