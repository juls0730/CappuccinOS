mod font;

use limine::FramebufferRequest;

pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new(0);

// This is slow, but significantly faster than filling the framebuffer pixel-by-pixel with for loops.
// idk, fix it later ig.
pub fn fill_screen(color: u32, framebuffer: Option<*mut u8>) {
    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response().get() {
        if framebuffer_response.framebuffer_count < 1 {
            return;
        }

        let framebuffer_response = &framebuffer_response.framebuffers()[0];
        let buffer_size = (framebuffer_response.pitch * framebuffer_response.height) as usize;
        let mut framebuffer_ptr = framebuffer_response.address.as_ptr().unwrap();

        if framebuffer.is_some() {
            framebuffer_ptr = framebuffer.unwrap()
        }

        crate::libs::util::memset32(framebuffer_ptr as *mut u32, color, buffer_size);
    }
}

pub fn put_char(character: char, cx: u16, cy: u16, fg: u32, bg: u32, framebuffer: Option<*mut u8>) {
    let font = font::G_8X16_FONT;

    let character_array = font[character as usize];

    for row in 0..character_array.len() {
        let character_byte = character_array[row as usize];
        for col in 0..8 {
            let pixel = (character_byte >> (7 - col)) & 0x01;

            let x = (cx * 8 + col) as u32;
            let y = (cy * 16 + row as u16) as u32;

            put_pixel(x, y, if pixel == 1 { fg } else { bg }, framebuffer);
        }
    }
}

pub fn put_pixel(x: u32, y: u32, color: u32, framebuffer: Option<*mut u8>) {
    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response().get() {
        if framebuffer_response.framebuffer_count < 1 {
            return;
        }

        let framebuffer_response = &framebuffer_response.framebuffers()[0];
        let mut framebuffer_ptr = framebuffer_response.address.as_ptr().unwrap();

        if framebuffer.is_some() {
            framebuffer_ptr = framebuffer.unwrap()
        }

        unsafe {
            let pixel_offset = (y * framebuffer_response.pitch as u32
                + (x * (framebuffer_response.bpp / 8) as u32))
                as isize;
            *(framebuffer_ptr.offset(pixel_offset) as *mut u32) = color;
        }
    }
}
