pub mod compressors;

use alloc::vec::Vec;
use limine::ModuleRequest;

pub static MODULE_REQUEST: ModuleRequest = ModuleRequest::new(0);

const SQUASHFS_INODE_OFFSET: fn(a: u32) -> u32 = |a| a & 0xFFF;
const SQUASHFS_INODE_BLK: fn(a: u32) -> u32 = |a| a >> 16;

pub fn init() {
    // TODO: Put the module request stuff in another file?
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
    // End TODO

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

    let squashfs = Squashfs::new(initramfs.base.as_ptr().unwrap());

    if squashfs.is_err() {
        crate::log_error!("Initramfs in corrupt!");
        return;
    }

    let squashfs = squashfs.unwrap();

    crate::println!("{:X?}", squashfs);
    crate::println!("{:?}", squashfs.superblock.features());

    squashfs.test();
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

impl Squashfs<'_> {
    fn new(ptr: *mut u8) -> Result<Squashfs<'static>, ()> {
        crate::println!("Parsing initramfs fs at {:p}", ptr);

        // bytes used from superblock
        let length = unsafe { u64::from_le(*(ptr.add(40) as *const u64)) as usize };

        let squashfs_data: &[u8] = unsafe { core::slice::from_raw_parts(ptr, length) };

        let superblock = SquashfsSuperblock::new(&squashfs_data)?;

        let data_table = &squashfs_data
            [core::mem::size_of::<SquashfsSuperblock>()..superblock.inode_table as usize];

        let inode_table =
            &squashfs_data[superblock.inode_table as usize..superblock.dir_table as usize];

        let directory_table =
            &squashfs_data[superblock.dir_table as usize..superblock.frag_table as usize];

        let mut fragment_table: Option<&[u8]> = None;

        if superblock.frag_table != u64::MAX {
            fragment_table = Some(
                &squashfs_data[superblock.frag_table as usize..superblock.export_table as usize],
            );
        }

        let mut export_table: Option<&[u8]> = None;

        if superblock.export_table != u64::MAX {
            export_table = Some(
                &squashfs_data[superblock.export_table as usize..superblock.id_table as usize],
            );
        }

        let mut id_table: &[u8] = &squashfs_data[superblock.id_table as usize..];
        let mut xattr_table: Option<&[u8]> = None;

        if superblock.xattr_table != u64::MAX {
            id_table =
                &squashfs_data[superblock.id_table as usize..superblock.xattr_table as usize];
            xattr_table = Some(&squashfs_data[superblock.xattr_table as usize..]);
        }

        return Ok(Squashfs {
            ptr: unsafe { ptr.add(core::mem::size_of::<SquashfsSuperblock>()) },
            superblock,
            data_table,
            inode_table,
            directory_table,
            fragment_table,
            export_table,
            id_table,
            xattr_table,
        });
    }

    // big function that does stuff hard coded-ly before I rip it all out
    pub fn test(&self) {
        // the bottom 15 bits, I think the last bit indicates whether the data is uncompressed
        let inode_table_header = u16::from_le_bytes(self.inode_table[0..2].try_into().unwrap());
        let inode_is_compressed = inode_table_header & 0x80 != 0;
        let inode_table_size = inode_table_header & 0x7FFF;

        if inode_table_size >= 8192 {
            panic!("Inode block is not less than 8KiB!");
        }

        let mut buffer: Vec<u8> = Vec::with_capacity(8192);

        if inode_is_compressed {
            todo!("uncompress zlib data")
        } else {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    self.inode_table.as_ptr().add(2),
                    buffer.as_mut_ptr(),
                    inode_table_size as usize,
                );

                buffer.set_len(inode_table_size as usize);
            }
        }

        let root_inode_block = self.superblock.root_inode_block as usize;
        let root_inode_offset = self.superblock.root_inode_offset as usize;

        let root_inode_header = self.read_inode(root_inode_offset as u32);

        let root_inode_header = InodeHeader {
            file_type: u16::from_le_bytes(
                buffer[root_inode_offset..root_inode_offset + 2]
                    .try_into()
                    .unwrap(),
            )
            .into(),
            _reserved: [
                u16::from_le_bytes(
                    buffer[root_inode_offset + 2..root_inode_offset + 4]
                        .try_into()
                        .unwrap(),
                ),
                u16::from_le_bytes(
                    buffer[root_inode_offset + 4..root_inode_offset + 6]
                        .try_into()
                        .unwrap(),
                ),
                u16::from_le_bytes(
                    buffer[root_inode_offset + 6..root_inode_offset + 8]
                        .try_into()
                        .unwrap(),
                ),
            ],
            mtime: u32::from_le_bytes(
                buffer[root_inode_offset + 8..root_inode_offset + 12]
                    .try_into()
                    .unwrap(),
            ),
            inode_num: u32::from_le_bytes(
                buffer[root_inode_offset + 12..root_inode_offset + 16]
                    .try_into()
                    .unwrap(),
            ),
        };

        crate::println!("{:X?}", root_inode_header);

        let root_inode = DirectoryInode {
            block_index: u32::from_le_bytes(
                buffer[root_inode_offset + 16..root_inode_offset + 20]
                    .try_into()
                    .unwrap(),
            ),
            link_count: u32::from_le_bytes(
                buffer[root_inode_offset + 20..root_inode_offset + 24]
                    .try_into()
                    .unwrap(),
            ),
            file_size: u16::from_le_bytes(
                buffer[root_inode_offset + 24..root_inode_offset + 26]
                    .try_into()
                    .unwrap(),
            ),
            block_offset: u16::from_le_bytes(
                buffer[root_inode_offset + 26..root_inode_offset + 28]
                    .try_into()
                    .unwrap(),
            ),
            parent_inode: u32::from_le_bytes(
                buffer[root_inode_offset + 28..root_inode_offset + 32]
                    .try_into()
                    .unwrap(),
            ),
        };

        crate::println!("{:?}", root_inode);
    }

    fn read_inode(&self, inode_num: u32) -> Option<InodeHeader> {
        let inode_offset = inode_num as usize + 2;

        if inode_offset + core::mem::size_of::<InodeHeader>() > self.inode_table.len() {
            return None;
        }

        let inode_header_bytes = &self.inode_table[inode_offset..(inode_offset + 16)];

        crate::println!("{:X?}", inode_header_bytes);
        let inode_header = InodeHeader::from_bytes(inode_header_bytes)?;

        Some(inode_header)
    }
}

#[derive(Debug)]
struct InodeHeader {
    file_type: InodeFileType,
    _reserved: [u16; 3],
    mtime: u32,
    inode_num: u32,
}

impl InodeHeader {
    fn from_bytes(bytes: &[u8]) -> Option<InodeHeader> {
        let file_type = u16::from_le_bytes(bytes[0..2].try_into().unwrap()).into();
        let mtime = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
        let inode_num = u32::from_le_bytes(bytes[12..16].try_into().unwrap());

        Some(InodeHeader {
            file_type,
            _reserved: [0; 3],
            mtime,
            inode_num,
        })
    }
}

#[derive(Debug)]
struct DirectoryInode {
    block_index: u32,
    link_count: u32,
    file_size: u16,
    block_offset: u16,
    parent_inode: u32,
}

#[derive(Debug)]
struct FileInode {
    block_start: u32,
    frag_idx: u32,
    block_offset: u32,
    file_size: u32,
    block_size: [u32; 0],
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InodeFileType {
    BasicDirectory = 1,
    BasicFile = 2,
    BasicSymlink = 3,
    BasicBlockDevice = 4,
    BasicCharDevice = 5,
    BasicPipe = 6,
    BasicSocked = 7,
    ExtendedDirectory = 8,
    ExtendedFile = 9,
    ExtendedSymlink = 10,
    ExtendedBlockDevice = 11,
    ExtendedPipe = 12,
    ExtendedSocked = 13,
}

impl Into<InodeFileType> for u16 {
    fn into(self) -> InodeFileType {
        match self {
            1 => InodeFileType::BasicDirectory,
            2 => InodeFileType::BasicFile,
            3 => InodeFileType::BasicSymlink,
            4 => InodeFileType::BasicBlockDevice,
            5 => InodeFileType::BasicCharDevice,
            6 => InodeFileType::BasicPipe,
            7 => InodeFileType::BasicSocked,
            8 => InodeFileType::ExtendedDirectory,
            9 => InodeFileType::ExtendedFile,
            10 => InodeFileType::ExtendedSymlink,
            11 => InodeFileType::ExtendedBlockDevice,
            12 => InodeFileType::ExtendedPipe,
            13 => InodeFileType::ExtendedSocked,
            _ => panic!("Unexpected Inode file type {self}!"),
        }
    }
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

#[repr(C)]
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
    root_inode_offset: u16,              //
    root_inode_block: u32,               //
    _reserved: u16,                      // 0x20
    bytes_used: u64,                     // 0x0103
    id_table: u64,                       // 0x00FB
    xattr_table: u64,                    // 0xFFFFFFFFFFFFFFFF
    inode_table: u64,                    // 0x7B
    dir_table: u64,                      // 0xA4
    frag_table: u64,                     // 0xD5
    export_table: u64,                   // 0xED
}

impl SquashfsSuperblock {
    fn new(bytes: &[u8]) -> Result<Self, ()> {
        let superblock = Self {
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
            root_inode_offset: u16::from_le_bytes(bytes[32..34].try_into().unwrap()),
            root_inode_block: u32::from_le_bytes(bytes[34..38].try_into().unwrap()),
            _reserved: u16::from_le_bytes(bytes[38..40].try_into().unwrap()),
            bytes_used: u64::from_le_bytes(bytes[40..48].try_into().unwrap()),
            id_table: u64::from_le_bytes(bytes[48..56].try_into().unwrap()),
            xattr_table: u64::from_le_bytes(bytes[56..64].try_into().unwrap()),
            inode_table: u64::from_le_bytes(bytes[64..72].try_into().unwrap()),
            dir_table: u64::from_le_bytes(bytes[72..80].try_into().unwrap()),
            frag_table: u64::from_le_bytes(bytes[80..88].try_into().unwrap()),
            export_table: u64::from_le_bytes(bytes[88..96].try_into().unwrap()),
        };

        if superblock.magic != 0x73717368 {
            return Err(());
        }

        if superblock.ver_major != 4 || superblock.ver_minor != 0 {
            return Err(());
        }

        if superblock.block_size > 1048576 {
            return Err(());
        }

        if superblock.block_log > 20 {
            return Err(());
        }

        if superblock.block_size != (1 << superblock.block_log) {
            return Err(());
        }

        if superblock.block_size == 0 {
            return Err(());
        }

        if ((superblock.block_size - 1) & superblock.block_size) != 0 {
            return Err(());
        }

        return Ok(superblock);
    }

    fn features(&self) -> SquashfsFeatures {
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
