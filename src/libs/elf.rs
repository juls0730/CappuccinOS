#[derive(Clone, Copy, Debug)]
pub struct Elf;

pub fn load_elf(bytes: &[u8]) -> Result<Elf, ()> {
    if &bytes[0..4] != b"\x74ELF" {
        return Err(());
    }

    if bytes[5] != 0x02 {
        // Only support 64-bit ELF files for now
        return Err(());
    }

    return Ok(Elf);
}
