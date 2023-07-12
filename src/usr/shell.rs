pub fn init_shell() {
	crate::drivers::keyboard::init_keyboard();

	prompt();
}

pub fn prompt() {
	super::tty::puts("> ");
}