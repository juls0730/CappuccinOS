use crate::libs::io::{inb, outb};

static VGA_AC_INDEX: u16 = 0x3C0;
static VGA_AC_WRITE: u16 = 0x3C0;
static VGA_AC_READ: u16 = 0x3C1;
static VGA_MISC_WRITE: u16 = 0x3C2;
static VGA_SEQ_INDEX: u16 =	0x3C4;
static VGA_SEQ_DATA: u16 = 0x3C5;
static VGA_DAC_READ_INDEX: u16 = 0x3C7;
static VGA_DAC_WRITE_INDEX: u16 =	0x3C8;
static VGA_DAC_DATA: u16 = 0x3C9;
static VGA_MISC_READ: u16 = 0x3CC;
static VGA_GC_INDEX: u16 = 0x3CE;
static VGA_GC_DATA: u16 = 0x3CF;
/*			COLOR emulation		MONO emulation */
static VGA_CRTC_INDEX: u16 = 0x3D4;		/* 0x3B4 */
static VGA_CRTC_DATA: u16 =	0x3D5;		/* 0x3B5 */
static VGA_INSTAT_READ: u16 =	0x3DA;

static VGA_NUM_SEQ_REGS: u16 = 5;
static VGA_NUM_CRTC_REGS: u16 = 25;
static VGA_NUM_GC_REGS: u16 = 9;
static VGA_NUM_AC_REGS: u16 = 21;
static VGA_NUM_REGS: u16 = 1 + VGA_NUM_SEQ_REGS + VGA_NUM_CRTC_REGS + VGA_NUM_GC_REGS + VGA_NUM_AC_REGS;

const VGA_BUFFER: *mut u8 = 0xA0000 as *mut u8;

pub fn init_vga() {
	// Set the registers for mode 12h
	let mut registers_720x480x16 = [
		/* MISC */
			0xE7,
		/* SEQ */
			0x03, 0x01, 0x08, 0x00, 0x06,
		/* CRTC */
			0x6B, 0x59, 0x5A, 0x82, 0x60, 0x8D, 0x0B, 0x3E,
			0x00, 0x40, 0x06, 0x07, 0x00, 0x00, 0x00, 0x00,
			0xEA, 0x0C, 0xDF, 0x2D, 0x08, 0xE8, 0x05, 0xE3,
			0xFF,
		/* GC */
			0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x05, 0x0F,
			0xFF,
		/* AC */
			0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
			0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
			0x01, 0x00, 0x0F, 0x00, 0x00,
		];

		write_regs(&mut registers_720x480x16);

		for x in 0..2 {
			put_pixel(x, 0, 9);
		}
}

fn put_pixel(mut pos_x: usize, pos_y: usize, vga_color: u8) {
	unsafe {
			let width_in_bytes = 720 / 8;

			let offset = width_in_bytes * pos_y + pos_x / 8;
			pos_x = (pos_x & 7) * 1;
			let mask: u8 = 0x80 >> pos_x;
			let mut pmask: u8 = 1;

			for plane in 0..4 {
				set_plane(plane);
				if pmask & vga_color != 0 {
					*VGA_BUFFER.offset(offset as isize) = *VGA_BUFFER.offset(offset as isize) | mask;
				} else {
					*VGA_BUFFER.offset(offset as isize) = *VGA_BUFFER.offset(offset as isize) & !mask;
				}
				pmask <<= 1;
			}
	}
}

fn set_plane(plane: u8) {
	let plane = plane & 3;
	let pmask = 1 << plane;

	/* set read plane */
	outb(VGA_GC_INDEX, 4);
	outb(VGA_GC_DATA, plane);
	
	/* set write plane */
	outb(VGA_SEQ_INDEX, 2);
	outb(VGA_SEQ_DATA, pmask);
}

fn write_regs(mut regs: &mut [u8]) {
	/* write MISCELLANEOUS reg */
	outb(VGA_MISC_WRITE, regs[0]);
	regs = &mut regs[1..];

	/* write SEQUENCER regs */
	for i in 0..VGA_NUM_SEQ_REGS {
		outb(VGA_SEQ_INDEX, i as u8);
		outb(VGA_SEQ_DATA, regs[0]);
		regs = &mut regs[1..];
	}

	/* unlock CRTC registers */
	outb(VGA_CRTC_INDEX, 0x03);
	outb(VGA_CRTC_DATA, inb(VGA_CRTC_DATA) | 0x80);
	outb(VGA_CRTC_INDEX, 0x11);
	outb(VGA_CRTC_DATA, inb(VGA_CRTC_DATA) & !0x80);

	/* make sure they remain unlocked */
	regs[0x03] |= 0x80;
	regs[0x11] &= !0x80;

	/* write CRTC regs */
	for i in 0..VGA_NUM_CRTC_REGS {
		outb(VGA_CRTC_INDEX, i as u8);
		outb(VGA_CRTC_DATA, regs[0]);
		regs = &mut regs[1..];
	}

	/* write GRAPHICS CONTROLLER regs */
	for i in 0..VGA_NUM_GC_REGS {
		outb(VGA_GC_INDEX, i as u8);
		outb(VGA_GC_DATA, regs[0]);
		regs = &mut regs[1..];
	}

	/* write ATTRIBUTE CONTROLLER regs */
	for i in 0..VGA_NUM_AC_REGS {
		// inb(VGA_INSTAT_READ);
		outb(VGA_AC_INDEX, i as u8);
		outb(VGA_AC_WRITE, regs[0]);
		regs = &mut regs[1..];
	}

	/* lock 16-color palette and unblank display */
	inb(VGA_INSTAT_READ);
	outb(VGA_AC_INDEX, 0x20);
}
