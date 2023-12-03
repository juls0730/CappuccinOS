#[derive(Debug)]
#[repr(u8)]
enum ZlibCompressionLevel {
    Fastest = 0,
    Fast,
    Default,
    Best,
}

impl Into<ZlibCompressionLevel> for u8 {
    fn into(self) -> ZlibCompressionLevel {
        match self {
            0 => ZlibCompressionLevel::Fastest,
            1 => ZlibCompressionLevel::Fast,
            2 => ZlibCompressionLevel::Default,
            3 => ZlibCompressionLevel::Best,
            _ => panic!("Unexpected compression level {self}"),
        }
    }
}

// ZLIB steam, see RFC 1950
pub fn uncompress_data(bytes: &[u8]) -> &[u8] {
    assert!(bytes.len() > 2);

    // Compression Method and flags
    let cmf = bytes[0];
    let flags = bytes[1];

    if cmf & 0x0F != 0x08 {
        panic!("Compression method is not GZIP!",);
    }

    let window_log2 = cmf >> 4 & 0x0F;

    if window_log2 > 0x07 {
        panic!("Unsupported window size {window_log2:X}!");
    }

    // TODO: Check if FCheck is valid

    let present_dictionary = flags >> 5 & 0x01 != 0;
    let compression_level: ZlibCompressionLevel = (flags >> 6 & 0x03).into();

    todo!("Uncompress data");
}
