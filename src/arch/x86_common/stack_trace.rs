use alloc::{borrow::ToOwned, string::String, vec::Vec};

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

        let instruction_pointer = unsafe { (*stackframe).rip };

        crate::print!("  {:#X} ", instruction_pointer);

        let instrcution_info = get_function_name(instruction_pointer);

        if let Ok((function_name, function_offset)) = instrcution_info {
            crate::println!("<{}+{:#X}>", function_name, function_offset);
        } else {
            crate::println!();
        }

        unsafe {
            stackframe = (*stackframe).back;
        };
    }
}

fn get_function_name(function_address: u64) -> Result<(String, u64), ()> {
    if crate::drivers::fs::vfs::VFS_INSTANCES.lock().read().len() == 0 {
        return Err(());
    }

    let vfs_lock = crate::drivers::fs::vfs::VFS_INSTANCES.lock();

    let symbols_fd = vfs_lock.read()[0].open("/boot/symbols.table")?;

    let symbols_table_bytes = symbols_fd.read()?;
    let symbols_table = core::str::from_utf8(&symbols_table_bytes).ok().ok_or(())?;

    let mut previous_symbol: Option<(&str, u64)> = None;

    let symbols_table_lines: Vec<&str> = symbols_table.lines().collect();

    for (i, line) in symbols_table_lines.iter().enumerate() {
        let line_parts: Vec<&str> = line.splitn(2, ' ').collect();

        if line_parts.len() < 2 {
            continue;
        }

        let (address, function_name) = (
            u64::from_str_radix(&line_parts[0], 16).ok().ok_or(())?,
            line_parts[1],
        );

        if address == function_address {
            return Ok((function_name.to_owned(), 0));
        }

        if i == symbols_table_lines.len() - 1 {
            return Ok((function_name.to_owned(), function_address - address));
        }

        if i == 0 {
            if function_address < address {
                return Err(());
            }

            previous_symbol = Some((function_name, address));
            continue;
        }

        if function_address > previous_symbol.unwrap().1 && function_address < address {
            // function is previous symbol
            return Ok((
                previous_symbol.unwrap().0.to_owned(),
                address - previous_symbol.unwrap().1,
            ));
        }

        previous_symbol = Some((function_name, address));
    }

    unreachable!();
}
