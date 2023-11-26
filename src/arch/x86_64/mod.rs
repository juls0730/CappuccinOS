pub mod interrupts;

// This inline is detremental to having readable stack traces
#[inline(always)]
pub fn push_gprs() {
    unsafe {
        core::arch::asm!(
            "push rax", "push rbx", "push rcx", "push rdx", "push rsi", "push rdi", "push rbp",
            "push r8", "push r9", "push r10", "push r11", "push r12", "push r13", "push r14",
            "push r15"
        );
    }
}

// This inline is detremental to having readable stack traces
#[inline(always)]
pub fn pop_gprs() {
    unsafe {
        core::arch::asm!(
            "pop rax", "pop rbx", "pop rcx", "pop rdx", "pop rsi", "pop rdi", "pop rbp", "pop r8",
            "pop r9", "pop r10", "pop r11", "pop r12", "pop r13", "pop r14", "pop r15",
        );
    }
}
