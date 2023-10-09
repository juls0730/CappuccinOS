mod font;

use crate::libs::mutex::Mutex;
use limine::FramebufferRequest;

pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new(0);

pub static FRAMEBUFFER: Mutex<Option<Framebuffer>> = Mutex::new(None);

// This is slow, but significantly faster than filling the framebuffer pixel-by-pixel with for loops.
// idk, fix it later ig.
pub fn fill_screen(color: u32, mirror_buffer: Option<Framebuffer>) {
    let framebuffer =
        get_framebuffer().expect("Tried to use framebuffer, but framebuffer was not found");

    let framebuffer_ptr = framebuffer.pointer;

    let buffer_size = (framebuffer.pitch / (framebuffer.bpp / 8)) * framebuffer.height;

    unsafe {
        if mirror_buffer.is_some() {
            crate::libs::util::memset32(
                mirror_buffer.unwrap().pointer as *mut u32,
                color,
                buffer_size,
            );
        }

        crate::libs::util::memset32(framebuffer_ptr as *mut u32, color, buffer_size);
    }
}

pub fn put_char(
    mut character: char,
    cx: u16,
    cy: u16,
    mut fg: u32,
    bg: u32,
    mirror_buffer: Option<Framebuffer>,
) {
    let font = font::G_8X16_FONT;

    if character as usize > u8::MAX as usize {
        character = '?';
        fg = 0xFF0000;
    }

    let character_array = font[character as usize];

    let framebuffer =
        get_framebuffer().expect("Tried to use framebuffer, but framebuffer was not found");

    let start_x = cx * 8;
    let start_y = cy * 16;

    for (row_idx, &character_byte) in character_array.iter().enumerate() {
        let mut byte = [bg; 8];
        for bit in 0..8 {
            byte[bit] = [bg, fg][((character_byte >> (7 - bit)) & 0b00000001) as usize]
        }

        let row_start_offset = (start_y as usize + row_idx) * framebuffer.pitch
            + (start_x as usize * framebuffer.bpp / 8);

        unsafe {
            let src_ptr = byte.as_ptr() as *const u128;

            core::ptr::copy_nonoverlapping(
                src_ptr,
                framebuffer.pointer.add(row_start_offset) as *mut u128,
                2,
            );

            if let Some(mirror_framebuffer) = mirror_buffer {
                core::ptr::copy_nonoverlapping(
                    src_ptr,
                    mirror_framebuffer.pointer.add(row_start_offset) as *mut u128,
                    2,
                );
            }
        };
    }
}

// pub static GLYPH_CACHE: Mutex<Option<alloc::vec::Vec<Option<[[u32; 8]; 16]>>>> = Mutex::new(None);

// pub fn put_char(
//     character: char,
//     cx: u16,
//     cy: u16,
//     fg: u32,
//     bg: u32,
//     mirror_buffer: Option<Framebuffer>,
// ) {
//     let font = font::G_8X16_FONT;
//     let character_array = font[character as usize];

//     let framebuffer =
//         get_framebuffer().expect("Tried to use framebuffer, but framebuffer was not found");

//     let glyph_index = character as u8 as usize;

//     if GLYPH_CACHE.lock().read().is_none() {
//         *GLYPH_CACHE.lock().write() = Some(alloc::vec![None; u8::MAX as usize]);
//     }

//     // Lock once and reuse the lock result
//     let mut glyph_cache_lock = GLYPH_CACHE.lock();
//     let glyph_cache = glyph_cache_lock.write().as_mut().unwrap();

//     if glyph_cache[glyph_index].is_none() {
//         let mut new_character_buf = [[bg; 8]; 16];

//         for (i, &character_byte) in character_array.iter().enumerate() {
//             let mut byte = [bg; 8];
//             for bit in 0..8 {
//                 byte[bit] = [bg, fg][((character_byte >> (7 - bit)) & 0b00000001) as usize];
//             }

//             new_character_buf[i] = byte;
//         }

//         glyph_cache[glyph_index] = Some(new_character_buf);
//     }

//     let start_x = cx * 8;
//     let start_y = cy * 16;

//     let character_buf = glyph_cache[glyph_index].unwrap();

//     for row_index in 0..character_buf.len() {
//         let row_num = start_y as usize + row_index;
//         let row_offset = (row_num as usize * framebuffer.pitch) as usize;
//         let col_offset = (start_x as usize * framebuffer.bpp / 8) as usize;
//         let row_start_offset = row_offset + col_offset;

//         unsafe {
//             let src_ptr = character_buf[row_index].as_ptr() as *const u128;

//             core::ptr::copy_nonoverlapping(
//                 src_ptr,
//                 framebuffer.pointer.add(row_start_offset) as *mut u128,
//                 2,
//             );

//             if let Some(mirror_framebuffer) = mirror_buffer {
//                 core::ptr::copy_nonoverlapping(
//                     src_ptr,
//                     mirror_framebuffer.pointer.add(row_start_offset) as *mut u128,
//                     2,
//                 );
//             }
//         };
//     }
// }

pub fn put_pixel(x: u32, y: u32, color: u32) {
    let framebuffer =
        get_framebuffer().expect("Tried to use framebuffer, but framebuffer was not found");

    let framebuffer_ptr = framebuffer.pointer;

    let pixel_offset = (y * framebuffer.pitch as u32 + (x * (framebuffer.bpp / 8) as u32)) as isize;

    unsafe {
        *(framebuffer_ptr.offset(pixel_offset) as *mut u32) = color;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub bpp: usize,
    pub pitch: usize,
    pub pointer: *mut u8,
}

impl Framebuffer {
    #[inline]
    const fn new(bpp: usize, pitch: usize, ptr: *mut u8, width: usize, height: usize) -> Self {
        return Self {
            width,
            height,
            bpp,
            pitch,
            pointer: ptr,
        };
    }
}

pub fn get_framebuffer() -> Option<Framebuffer> {
    let framebuffer_mutex_lock = FRAMEBUFFER.lock();

    if framebuffer_mutex_lock.read().is_some() {
        return Some(FRAMEBUFFER.lock().read().unwrap());
    }

    let framebuffer_response = crate::drivers::video::FRAMEBUFFER_REQUEST
        .get_response()
        .get();

    if framebuffer_response.is_none() {
        return None;
    }

    // eww, variable redeclaration
    let framebuffer_response = framebuffer_response.unwrap();
    if framebuffer_response.framebuffer_count < 1 {
        return None;
    }

    let framebuffer_response = &framebuffer_response.framebuffers()[0];

    let framebuffer = Framebuffer::new(
        framebuffer_response.bpp as usize,
        framebuffer_response.pitch as usize,
        framebuffer_response.address.as_ptr().unwrap() as *mut u8,
        framebuffer_response.width as usize,
        framebuffer_response.height as usize,
    );

    let mut framebuffer_mutex_lock = FRAMEBUFFER.lock();

    *(framebuffer_mutex_lock.write()) = Some(framebuffer);

    return Some(framebuffer);
}
