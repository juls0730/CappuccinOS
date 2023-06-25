use crate::libs::io::{inb, outb};

pub static PORT: u16 = 0x3f8;

pub fn init_serial() -> u8 {
	outb(PORT + 1, 0x00);
	outb(PORT + 3, 0x80);
	outb(PORT + 0, 0x03);
	outb(PORT + 1, 0x00);
	outb(PORT + 3, 0x03);
	outb(PORT + 2, 0xC7);
	outb(PORT + 4, 0x0B);
	outb(PORT + 4, 0x1E);
	outb(PORT + 0, 0xAE);

  // Check if serial is faulty
  if inb(PORT + 0) != 0xAE {
      return 1;
  }
	
  // Set serial in normal operation mode
  outb(PORT + 4, 0x0F);
	return 0;
}

fn is_transmit_empty() -> bool {
	return (inb(PORT + 5) & 0x20) != 0x20;
}

pub fn write_serial(a: u8) {
		while is_transmit_empty() {}
    outb(PORT, a);
}