#[repr(C, packed)]
struct VbeModeInfoStructure {
    attributes: u16, // deprecated, only bit 7 should be of interest to you, and it indicates the mode supports a linear frame buffer.
    window_a: u8,    // deprecated
    window_b: u8,    // deprecated
    granularity: u16, // deprecated; used while calculating bank numbers
    window_size: u16,
    segment_a: u16,
    segment_b: u16,
    win_func_ptr: u32, // deprecated; used to switch banks from protected mode without returning to real mode
    pitch: u16,        // number of bytes per horizontal line
    width: u16,        // width in pixels
    height: u16,       // height in pixels
    w_char: u8,        // unused...
    y_char: u8,        // ...
    planes: u8,
    bpp: u8,   // bits per pixel in this mode
    banks: u8, // deprecated; total number of banks in this mode
    memory_model: u8,
    bank_size: u8, // deprecated; size of a bank, almost always 64 KB but may be 16 KB...
    image_pages: u8,
    reserved0: u8,

    red_mask: u8,
    red_position: u8,
    green_mask: u8,
    green_position: u8,
    blue_mask: u8,
    blue_position: u8,
    reserved_mask: u8,
    reserved_position: u8,
    direct_color_attributes: u8,

    framebuffer: u32, // physical address of the linear frame buffer; write here to draw to the screen
    off_screen_mem_off: u32,
    off_screen_mem_size: u16, // size of memory in the framebuffer but not being displayed on the screen
    reserved1: [u8; 206],
}

const VBE_INFO_ADDR: usize = 0x8000;

pub fn init_video() {
	fill_screen(0xFFFFFF);
}

pub fn fill_screen(color: u32) {
	for x in 0..1024 {
		for y in 0..768 {
			put_pixel(x, y, color);
		}
	}
}

fn put_pixel(x: u32, y: u32, color: u32) {
	let mut g_vbe: *mut VbeModeInfoStructure = VBE_INFO_ADDR as *mut VbeModeInfoStructure;
	g_vbe = unsafe { &mut *g_vbe };

	unsafe {
		let pixel_offset: *mut u32 = (y * (*g_vbe).pitch as u32 + (x * ((*g_vbe).bpp/8) as u32) + (*g_vbe).framebuffer) as *mut u32;
		*pixel_offset = color;
	}
}