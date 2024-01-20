use alloc::vec::Vec;

use crate::{
    drivers::fs::{initramfs::INITRAMFS, vfs::VfsFileSystem},
    libs::{lazy::Lazy, mutex::Mutex},
};

#[derive(Debug)]
pub struct PSFFontHeader {
    pub magic: u32,
    pub length: u32,
    pub bytes_per_glyph: u32,
    pub width: u32,
    pub height: u32,
}

pub struct PSFFont {
    pub header: PSFFontHeader,
    pub data: Vec<Vec<u8>>,
}

impl PSFFont {
    fn from_file_data(file_data: Vec<u8>) -> Result<PSFFont, ()> {
        let header = PSFFontHeader {
            magic: u32::from_be_bytes(file_data[0..4].try_into().unwrap()),
            length: u32::from_le_bytes(file_data[16..20].try_into().unwrap()),
            bytes_per_glyph: u32::from_le_bytes(file_data[20..24].try_into().unwrap()),
            height: u32::from_le_bytes(file_data[24..28].try_into().unwrap()),
            width: u32::from_le_bytes(file_data[28..32].try_into().unwrap()),
        };

        if header.magic != 0x72B54A86 {
            return Err(());
        }

        let data: Vec<_> = file_data[32..]
            .chunks_exact(header.bytes_per_glyph as usize)
            .map(Vec::from)
            .collect();

        Ok(PSFFont { header, data })
    }
}

pub static FONT: Lazy<PSFFont> = Lazy::new(|| {
    let file_data = INITRAMFS
        .open("/font.psf")
        .unwrap()
        .read()
        .unwrap()
        .to_vec();

    PSFFont::from_file_data(file_data).expect("Failed to create terminal font!")
});
