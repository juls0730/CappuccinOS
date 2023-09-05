#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => (crate::println!("\033[97m[ \033[90m? \033[97m]\033[37m {}", &alloc::format!($($arg)*)));
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => (crate::println!("\033[97m[ \033[91m! \033[97m]\033[37m {}", &alloc::format!($($arg)*)));
}

#[macro_export]
macro_rules! log_ok {
    ($($arg:tt)*) => (crate::println!("\033[97m[ \033[92m* \033[97m]\033[37m {}", &alloc::format!($($arg)*)));
}
