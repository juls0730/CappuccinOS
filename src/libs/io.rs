use core::arch::asm;

#[inline]
pub fn outb(port: u16, value: u8) {
	unsafe {
		asm!(
			"out dx, al", 
			in("dx") port,
			in("al") value,  
			options(preserves_flags, nomem, nostack)
		);
	}
	return;
}

#[inline]
pub fn inb(port: u16) -> u8 {
	let mut value: u8;
	unsafe {
		asm!(
			"in al, dx", 
			out("al") value, 
			in("dx") port, 
			options(preserves_flags, nomem, nostack)
		);
	}
	return value;
}

#[inline]
pub fn outw(port: u16, value: u16) {
	unsafe {
		asm!(
			"out dx, eax", 
			in("dx") port,
			in("eax") value,  
			options(preserves_flags, nomem, nostack)
		);
	}
	return;
}

#[inline]
pub fn inw(port: u16) -> u16 {
	let mut value: u16;
	unsafe {
		asm!(
			"in eax, dx",
			out("eax") value, 
			in("dx") port, 
			options(preserves_flags, nomem, nostack)
		);
	}
	return value;
}

