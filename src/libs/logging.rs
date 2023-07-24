use crate::usr::tty::{puts, CURSOR};

pub fn log_info(msg: &str) {
    puts("\033[97m[ \033[90m* \033[97m]\033[37m  ");
    puts(msg);
}

pub fn log_error(msg: &str) {
    puts("\033[97m[ \033[91m* \033[97m]\033[37m  ");
    puts(msg)
}

pub fn log_ok(msg: &str) {
    puts("\033[97m[ \033[92m* \033[97m]\033[37m  ");
    puts(msg);
}
