pub fn init_shell() {
    crate::drivers::keyboard::init_keyboard(super::tty::handle_key);

    prompt();
}

pub fn prompt() {
    super::tty::puts("> ");
}
