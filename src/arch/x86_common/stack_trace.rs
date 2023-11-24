#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct StackFrame {
    back: *const StackFrame,
    rip: u64,
}

pub fn print_stack_trace(max_frames: usize) {
    let mut stackframe: *const StackFrame;

    unsafe {
        core::arch::asm!("mov {0:r}, rbp", out(reg) stackframe);
    };

    crate::println!("Stack Trace:");
    for _frame in 0..max_frames {
        if stackframe.is_null() || unsafe { (*stackframe).back.is_null() } {
            break;
        }

        unsafe {
            crate::println!("  {:#X}", (*stackframe).rip);
            stackframe = (*stackframe).back;
        };
    }
}
