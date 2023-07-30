pub fn init_shell() {
    crate::drivers::keyboard::init_keyboard(handle_key);

    prompt();
}

fn handle_key(key: crate::drivers::keyboard::Key, mods: crate::drivers::keyboard::ModStatuses) {
    super::tty::handle_key(key, mods);
}

pub fn prompt() {
    super::tty::puts("> ");
}
