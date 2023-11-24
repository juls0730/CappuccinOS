pub mod io;
pub mod pic;
pub mod stack_trace;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub fn pause() {
    unsafe {
        core::arch::asm!("pause");
    };
}
