use crate::println;

pub fn log_info(msg: &str) {
    println!("\033[97m[ \033[90m? \033[97m]\033[37m  {}", msg);
}

pub fn log_error(msg: &str) {
    println!("\033[97m[ \033[91m! \033[97m]\033[37m  {}", msg);
}

pub fn log_ok(msg: &str) {
    println!("\033[97m[ \033[92m* \033[97m]\033[37m  {}", msg);
}
