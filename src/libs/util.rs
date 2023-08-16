pub fn memset32(dst: *mut u32, val: u32, count: usize) -> *mut u32 {
    let mut buf = dst;

    unsafe {
        while buf < dst.offset(count as isize) {
            core::ptr::write_volatile(buf, val);
            buf = buf.offset(1);
        }
    }

    return dst;
}
