use crate::usr::tty::puts;
use alloc::format;

pub fn log_info(msg: &str) {
    puts(&format!("\033[97m[ \033[90m* \033[97m]\033[37m  {}\n", msg));
}

pub fn log_error(msg: &str) {
    puts(&format!("\033[97m[ \033[91m* \033[97m]\033[37m  {}\n", msg));
}

pub fn log_ok(msg: &str) {
    puts(&format!("\033[97m[ \033[92m* \033[97m]\033[37m  {}\n", msg));
}
