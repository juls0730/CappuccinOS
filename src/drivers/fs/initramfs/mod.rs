pub mod compressors;

use limine::ModuleRequest;

pub static MODULE_REQUEST: ModuleRequest = ModuleRequest::new(0);

pub fn init() {
    // TODO: Put this stuff in another file?
    if MODULE_REQUEST.get_response().get().is_none() {
        crate::log_error!("Module request in none!");
        return;
    }
    let module_response = MODULE_REQUEST.get_response().get().unwrap();

    let mut initramfs = None;

    let module_name = "initramfs.img";

    for module in module_response.modules() {
        let c_path = module.path.to_str();
        if c_path.is_none() {
            continue;
        }

        if !c_path.unwrap().to_str().unwrap().contains(module_name) {
            continue;
        }

        initramfs = Some(module);
    }

    if initramfs.is_none() {
        crate::log_error!("Initramfs was not found!");
        return;
    }
    let initramfs = initramfs.unwrap();

    crate::println!("Initramfs is located at: {:#018X?}", unsafe {
        initramfs.base.as_ptr().unwrap()
            ..initramfs
                .base
                .as_ptr()
                .unwrap()
                .add(initramfs.length as usize)
    });

    init_fs(initramfs.base.as_ptr().unwrap());
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
enum SquashfsCompressionType {
    GZIP = 1,
    LZMA = 2,
    LZO = 3,
    XZ = 4,
    LZ4 = 5,
    ZSTD = 6,
}

#[repr(u16)]
enum SquashfsFlags {
    UncompressedInodes = 0x0001,
    UncompressedDataBlocks = 0x0002,
    Reserved = 0x0004,
    UncompressedFragments = 0x0008,
    UnusedFragments = 0x0010,
    FragmentsAlwaysPresent = 0x0020,
    DeduplicatedData = 0x0040,
    PresentNFSTable = 0x0080,
    UncompressedXattrs = 0x0100,
    NoXattrs = 0x0200,
    PresentCompressorOptions = 0x0400,
    UncompressedIDTable = 0x0800,
}

#[derive(Debug)]
struct SquashfsFeatures {
    uncompressed_inodes: bool,
    uncompressed_data_blocks: bool,
    _reserved: bool,
    uncompressed_fragments: bool,
    unused_fragments: bool,
    fragments_always_present: bool,
    deduplicated_data: bool,
    nfs_table_present: bool,
    uncompressed_xattrs: bool,
    no_xattrs: bool,
    compressor_options_present: bool,
    uncompressed_id_table: bool,
}

impl Into<SquashfsCompressionType> for u16 {
    fn into(self) -> SquashfsCompressionType {
        match self {
            1 => SquashfsCompressionType::GZIP,
            2 => SquashfsCompressionType::LZMA,
            3 => SquashfsCompressionType::LZO,
            4 => SquashfsCompressionType::XZ,
            5 => SquashfsCompressionType::LZ4,
            6 => SquashfsCompressionType::ZSTD,
            _ => panic!("Unexpected Squashfs compression type!"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct SquashfsSuperblock {
    magic: u32,                          // 0x73717368
    inode_count: u32,                    // 0x02
    mod_time: u32,                       // varies
    block_size: u32,                     // 0x20000
    frag_count: u32,                     // 0x01
    compressor: SquashfsCompressionType, // GZIP
    block_log: u16,                      // 0x11
    flags: u16,                          // 0xC0
    id_count: u16,                       // 0x01
    ver_major: u16,                      // 0x04
    ver_minor: u16,                      // 0x00
    root_inode: u64,                     // 0x20
    bytes_used: u64,                     // 0x0103
    id_table: u64,                       // 0x00FB
    xattr_table: u64,                    // 0xFFFFFFFFFFFFFFFF
    inode_table: u64,                    // 0x7B
    dir_table: u64,                      // 0xA4
    frag_table: u64,                     // 0xD5
    export_table: u64,                   // 0xED
}

impl SquashfsSuperblock {
    fn new(bytes: &[u8]) -> Self {
        return Self {
            magic: u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            inode_count: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            mod_time: u32::from_le_bytes(bytes[8..12].try_into().unwrap()),
            block_size: u32::from_le_bytes(bytes[12..16].try_into().unwrap()),
            frag_count: u32::from_le_bytes(bytes[16..20].try_into().unwrap()),
            compressor: u16::from_le_bytes(bytes[20..22].try_into().unwrap()).into(),
            block_log: u16::from_le_bytes(bytes[22..24].try_into().unwrap()),
            flags: u16::from_le_bytes(bytes[24..26].try_into().unwrap()),
            id_count: u16::from_le_bytes(bytes[26..28].try_into().unwrap()),
            ver_major: u16::from_le_bytes(bytes[28..30].try_into().unwrap()),
            ver_minor: u16::from_le_bytes(bytes[30..32].try_into().unwrap()),
            root_inode: u64::from_le_bytes(bytes[32..40].try_into().unwrap()),
            bytes_used: u64::from_le_bytes(bytes[40..48].try_into().unwrap()),
            id_table: u64::from_le_bytes(bytes[48..56].try_into().unwrap()),
            xattr_table: u64::from_le_bytes(bytes[56..64].try_into().unwrap()),
            inode_table: u64::from_le_bytes(bytes[64..72].try_into().unwrap()),
            dir_table: u64::from_le_bytes(bytes[72..80].try_into().unwrap()),
            frag_table: u64::from_le_bytes(bytes[80..88].try_into().unwrap()),
            export_table: u64::from_le_bytes(bytes[88..96].try_into().unwrap()),
        };
    }

    fn features(&self) -> SquashfsFeatures {
        // let graphical_output = ((*self.feature_bits.lock().read()) & 0x01) != 0;
        // let serial_output = ((*self.feature_bits.lock().read()) & 0x02) != 0;
        // let doubled_buffered = ((*self.feature_bits.lock().read()) & 0x04) != 0;
        let uncompressed_inodes = (self.flags & SquashfsFlags::UncompressedInodes as u16) != 0;
        let uncompressed_data_blocks =
            (self.flags & SquashfsFlags::UncompressedDataBlocks as u16) != 0;
        let _reserved = (self.flags & SquashfsFlags::Reserved as u16) != 0;
        let uncompressed_fragments =
            (self.flags & SquashfsFlags::UncompressedFragments as u16) != 0;
        let unused_fragments = (self.flags & SquashfsFlags::UnusedFragments as u16) != 0;
        let fragments_always_present =
            (self.flags & SquashfsFlags::FragmentsAlwaysPresent as u16) != 0;
        let deduplicated_data = (self.flags & SquashfsFlags::DeduplicatedData as u16) != 0;
        let nfs_table_present = (self.flags & SquashfsFlags::PresentNFSTable as u16) != 0;
        let uncompressed_xattrs = (self.flags & SquashfsFlags::UncompressedXattrs as u16) != 0;
        let no_xattrs = (self.flags & SquashfsFlags::NoXattrs as u16) != 0;
        let compressor_options_present =
            (self.flags & SquashfsFlags::PresentCompressorOptions as u16) != 0;
        let uncompressed_id_table = (self.flags & SquashfsFlags::UncompressedIDTable as u16) != 0;

        return SquashfsFeatures {
            uncompressed_inodes,
            uncompressed_data_blocks,
            _reserved,
            uncompressed_fragments,
            unused_fragments,
            fragments_always_present,
            deduplicated_data,
            nfs_table_present,
            uncompressed_xattrs,
            no_xattrs,
            compressor_options_present,
            uncompressed_id_table,
        };
    }
}

#[derive(Debug)]
struct Squashfs<'a> {
    ptr: *mut u8,
    superblock: SquashfsSuperblock,
    data_table: &'a [u8],
    inode_table: &'a [u8],
    directory_table: &'a [u8],
    fragment_table: Option<&'a [u8]>,
    export_table: Option<&'a [u8]>,
    id_table: &'a [u8],
    xattr_table: Option<&'a [u8]>,
}

fn init_fs(initramfs: *mut u8) {
    crate::println!("Parsing initramfs fs at {:p}", initramfs);

    // bytes used from superblock
    let length = unsafe { u64::from_le(*(initramfs.add(40) as *const u64)) as usize };

    let squashfs_data: &[u8] = unsafe { core::slice::from_raw_parts(initramfs, length) };

    let superblock = SquashfsSuperblock::new(&squashfs_data);

    let data_table =
        &squashfs_data[core::mem::size_of::<SquashfsSuperblock>()..superblock.inode_table as usize];

    let inode_table =
        &squashfs_data[superblock.inode_table as usize..superblock.dir_table as usize];

    let directory_table =
        &squashfs_data[superblock.dir_table as usize..superblock.frag_table as usize];

    let mut fragment_table: Option<&[u8]> = None;

    if superblock.frag_table != u64::MAX {
        fragment_table =
            Some(&squashfs_data[superblock.frag_table as usize..superblock.export_table as usize]);
    }

    let mut export_table: Option<&[u8]> = None;

    if superblock.export_table != u64::MAX {
        export_table =
            Some(&squashfs_data[superblock.export_table as usize..superblock.id_table as usize]);
    }

    let mut id_table: &[u8] = &squashfs_data[superblock.id_table as usize..];
    let mut xattr_table: Option<&[u8]> = None;

    if superblock.xattr_table != u64::MAX {
        id_table = &squashfs_data[superblock.id_table as usize..superblock.xattr_table as usize];
        xattr_table = Some(&squashfs_data[superblock.xattr_table as usize..]);
    }

    let squashfs = Squashfs {
        ptr: initramfs,
        superblock,
        data_table,
        inode_table,
        directory_table,
        fragment_table,
        export_table,
        id_table,
        xattr_table,
    };

    crate::println!("{:#X?}", squashfs);

    crate::println!("{:?}", squashfs.superblock.features());
}
