use crate::drivers::video::{puts, set_color};

pub fn log_info(msg: &str) {
	log_indicator(0xcacaca);
	set_color(0xbababa);
	puts(msg);
}

pub fn log_error(msg: &str) {
	log_indicator(0xD90202);
	set_color(0xbababa);
	puts(msg)
}

pub fn log_ok(msg: &str) {
	log_indicator(0x4AF626);
	set_color(0xbababa);
	puts(msg);
}

// Do stupid things to print brackets with a colored asterisk in the center
fn log_indicator(indicator_color: u32) {
	set_color(0xffffff);
	puts("[ ");
	set_color(indicator_color);
	puts("*");
	set_color(0xffffff);
	puts(" ]  ");
}