pub mod compressors;

use core::fmt::{self, Debug};

use alloc::{boxed::Box, sync::Arc, vec::Vec};
use limine::ModuleRequest;

use crate::libs::cell::LazyCell;

use super::vfs::{FsOps, VNode, VNodeOperations};

pub static MODULE_REQUEST: ModuleRequest = ModuleRequest::new(0);

// TODO: do something better than this shite
pub static INITRAMFS: LazyCell<Squashfs> = LazyCell::new(init);

fn init() -> Squashfs<'static> {
    // TODO: Put the module request stuff in another file?
    if MODULE_REQUEST.get_response().get().is_none() {
        panic!("Module request in none!");
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
        panic!("Initramfs was not found!");
    }
    let initramfs = initramfs.unwrap();

    let squashfs = Squashfs::new(initramfs.base.as_ptr().unwrap());

    if squashfs.is_err() {
        panic!("Initramfs in corrupt!");
    }

    let squashfs = squashfs.unwrap();

    return squashfs;
}

#[repr(C)]
#[derive(Debug)]
pub struct Squashfs<'a> {
    superblock: SquashfsSuperblock,
    start: *mut u8,
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
        // crate::log_info!("Parsing initramfs at {:p}", ptr);

        // 40 is the offset for bytes used by the archive in the superblock
        let length = unsafe { u64::from_le(*(ptr.add(40) as *const u64)) as usize };

        let squashfs_data: &[u8] = unsafe { core::slice::from_raw_parts(ptr, length) };

        let superblock = SquashfsSuperblock::new(squashfs_data)?;

        let data_table = &squashfs_data
            [core::mem::size_of::<SquashfsSuperblock>()..superblock.inode_table as usize];

        macro_rules! get_metadata_table {
            ($table_type:ident) => {{
                // get table size minus the top bit (indicates compression) plus 2, because
                // The table size is minus the size of the size header (two bytes)
                let table_size = (u16::from_le_bytes(
                    squashfs_data
                        [superblock.$table_type as usize..superblock.$table_type as usize + 2]
                        .try_into()
                        .unwrap(),
                ) & 0x7FFF)
                    + 2;

                &squashfs_data[superblock.$table_type as usize
                    ..superblock.$table_type as usize + table_size as usize]
            }};
        }

        let inode_table = get_metadata_table!(inode_table);

        let directory_table = get_metadata_table!(dir_table);

        let fragment_table: Option<&[u8]> = {
            if superblock.frag_table == u64::MAX {
                None
            } else if superblock.export_table != u64::MAX {
                Some(
                    &squashfs_data
                        [superblock.frag_table as usize..superblock.export_table as usize],
                )
            } else if superblock.xattr_table != u64::MAX {
                Some(
                    &squashfs_data[superblock.frag_table as usize..superblock.xattr_table as usize],
                )
            } else {
                Some(&squashfs_data[superblock.frag_table as usize..superblock.id_table as usize])
            }
        };

        let export_table: Option<&[u8]> = {
            if superblock.export_table == u64::MAX {
                None
            } else if superblock.xattr_table != u64::MAX {
                Some(
                    &squashfs_data
                        [superblock.export_table as usize..superblock.xattr_table as usize],
                )
            } else {
                Some(&squashfs_data[superblock.export_table as usize..superblock.id_table as usize])
            }
        };

        let id_table: &[u8] = if superblock.xattr_table != u64::MAX {
            &squashfs_data[superblock.id_table as usize..superblock.xattr_table as usize]
        } else {
            &squashfs_data[superblock.id_table as usize..]
        };

        let xattr_table: Option<&[u8]> = {
            if superblock.xattr_table == u64::MAX {
                None
            } else {
                Some(&squashfs_data[superblock.xattr_table as usize..])
            }
        };

        return Ok(Squashfs {
            superblock,
            start: ptr,
            data_table,
            inode_table,
            directory_table,
            fragment_table,
            export_table,
            id_table,
            xattr_table,
        });
    }

    fn read_root_dir(&self) -> BasicDirectoryInode {
        let root_inode_offset = self.superblock.root_inode_offset as usize;

        let root_inode: BasicDirectoryInode = self
            .read_inode(root_inode_offset as u32)
            .try_into()
            .expect("Failed to try_into");

        return root_inode;
    }

    fn read_inode(&self, inode_num: u32) -> Inode {
        let inode_table = &self.get_decompressed_table(self.inode_table, (true, None));

        let inode_offset = inode_num as usize;

        let inode_file_type: InodeFileType = u16::from_le_bytes(
            inode_table[inode_offset..(inode_offset + 2)]
                .try_into()
                .unwrap(),
        )
        .into();

        match inode_file_type {
            InodeFileType::BasicDirectory => {
                return Inode::BasicDirectory(BasicDirectoryInode::from_bytes(
                    &inode_table[inode_offset..],
                ));
            }
            InodeFileType::BasicFile => {
                return Inode::BasicFile(BasicFileInode::from_bytes(&inode_table[inode_offset..]));
            }
            _ => panic!("Unsupported or unknown inode file type {inode_file_type:?}!"),
        };
    }

    // metadata_block takes a tuple, the first element is whether the array is a metadata block,
    // and the second element is a is_compressed override if the array is not a metadata block.
    fn get_decompressed_table(
        &self,
        table: &[u8],
        metadata_block: (bool, Option<bool>),
    ) -> Vec<u8> {
        // the bottom 15 bits, I think the last bit indicates whether the data is uncompressed
        let header = u16::from_le_bytes(table[0..2].try_into().unwrap());
        let table_is_compressed = if !metadata_block.0 {
            metadata_block.1.unwrap()
        } else {
            header & 0x8000 == 0
        };
        // let table_size = header & 0x7FFF;

        // if table.len() >= 8192 {
        //     panic!("Inode block is not less than 8KiB!");
        // }

        let mut buffer: Vec<u8> = Vec::new();
        let bytes = if metadata_block.0 { &table[2..] } else { table };

        if table_is_compressed {
            match self.superblock.compressor {
                SquashfsCompressionType::Gzip => {
                    buffer.extend_from_slice(&compressors::gzip::uncompress_data(bytes).unwrap());
                }
                _ => {
                    crate::println!("Unsupported compression type")
                }
            }
        } else {
            buffer.extend(bytes);
        }

        return buffer;
    }
}

impl<'a> FsOps for Squashfs<'a> {
    fn mount(&self, path: &str, data: &mut *mut u8, vfsp: *const super::vfs::Vfs) {
        // STUB
    }

    fn unmount(&self, vfsp: *const super::vfs::Vfs) {
        // STUB
    }

    fn root(&self, vfsp: *const super::vfs::Vfs) -> super::vfs::VNode {
        let root_dir = Inode::BasicDirectory(self.read_root_dir());

        super::vfs::VNode {
            flags: 0,
            ref_count: 0,
            shared_lock_count: 0,
            exclusive_lock_count: 0,
            vfs_mounted_here: None,
            ops: Box::new(Inode::BasicDirectory(self.read_root_dir())),
            node_data: None,
            parent: vfsp,
            typ: super::vfs::VNodeType::Directory,
            data: core::ptr::null_mut(),
        }
    }

    fn fid(&self, path: &str, vfsp: *const super::vfs::Vfs) -> Option<super::vfs::FileId> {
        todo!();
    }

    fn statfs(&self, vfsp: *const super::vfs::Vfs) -> super::vfs::StatFs {
        todo!();
    }

    fn sync(&self, vfsp: *const super::vfs::Vfs) {
        todo!();
    }

    fn vget(&self, fid: super::vfs::FileId, vfsp: *const super::vfs::Vfs) -> super::vfs::VNode {
        todo!();
    }
}

// impl<'a> VfsFileSystem for Squashfs<'a> {
//     fn open(&self, path: &str) -> Result<Box<dyn VfsFile + '_>, ()> {
//         let path_components: Vec<&str> = path.trim_start_matches('/').split('/').collect();
//         let mut current_dir = self.read_root_dir();

//         for (i, &part) in path_components.iter().enumerate() {
//             let file = current_dir.find(part).ok_or(())?;

//             match file {
//                 Inode::BasicDirectory(dir) => {
//                     current_dir = dir;
//                 }
//                 Inode::BasicFile(file) => {
//                     if i < path_components.len() - 1 {
//                         return Err(());
//                     }

//                     return Ok(Box::new(file));
//                 }
//             }
//         }

//         return Err(());
//     }

//     fn read_dir(&self, _path: &str) -> Result<Box<dyn VfsDirectory>, ()> {
//         unimplemented!()
//     }
// }

#[derive(Clone, Copy, Debug)]
enum Inode {
    BasicFile(BasicFileInode),
    BasicDirectory(BasicDirectoryInode),
}

impl VNodeOperations for Inode {
    fn open(&self, f: u32, c: super::vfs::UserCred, vp: *const VNode) -> Result<Arc<[u8]>, ()> {
        todo!()
    }

    fn close(&self, f: u32, c: super::vfs::UserCred, vp: *const VNode) {
        todo!()
    }

    fn rdwr(
        &self,
        uiop: *const super::vfs::UIO,
        direction: super::vfs::IODirection,
        f: u32,
        c: super::vfs::UserCred,
        vp: *const VNode,
    ) {
        todo!()
    }

    fn ioctl(&self, com: u32, d: *mut u8, f: u32, c: super::vfs::UserCred, vp: *const VNode) {
        todo!()
    }

    fn select(&self, w: super::vfs::IODirection, c: super::vfs::UserCred, vp: *const VNode) {
        todo!()
    }

    fn getattr(&self, c: super::vfs::UserCred, vp: *const VNode) -> super::vfs::VAttr {
        todo!()
    }

    fn setattr(&self, va: super::vfs::VAttr, c: super::vfs::UserCred, vp: *const VNode) {
        todo!()
    }

    fn access(&self, m: u32, c: super::vfs::UserCred, vp: *const VNode) {
        todo!()
    }

    fn lookup(
        &self,
        nm: &str,
        c: super::vfs::UserCred,
        vp: *const VNode,
    ) -> Result<super::vfs::VNode, ()> {
        todo!()
    }

    fn create(
        &self,
        nm: &str,
        va: super::vfs::VAttr,
        e: u32,
        m: u32,
        c: super::vfs::UserCred,
        vp: *const VNode,
    ) -> Result<super::vfs::VNode, ()> {
        todo!()
    }

    fn link(
        &self,
        target_dir: *mut super::vfs::VNode,
        target_name: &str,
        c: super::vfs::UserCred,
        vp: *const VNode,
    ) {
        todo!()
    }

    fn rename(
        &self,
        nm: &str,
        target_dir: *mut super::vfs::VNode,
        target_name: &str,
        c: super::vfs::UserCred,
        vp: *const VNode,
    ) {
        todo!()
    }

    fn mkdir(
        &self,
        nm: &str,
        va: super::vfs::VAttr,
        c: super::vfs::UserCred,
        vp: *const VNode,
    ) -> Result<super::vfs::VNode, ()> {
        todo!()
    }

    fn readdir(&self, uiop: *const super::vfs::UIO, c: super::vfs::UserCred, vp: *const VNode) {
        todo!()
    }

    fn symlink(
        &self,
        link_name: &str,
        va: super::vfs::VAttr,
        target_name: &str,
        c: super::vfs::UserCred,
        vp: *const VNode,
    ) {
        todo!()
    }

    fn readlink(&self, uiop: *const super::vfs::UIO, c: super::vfs::UserCred, vp: *const VNode) {
        todo!()
    }

    fn fsync(&self, c: super::vfs::UserCred, vp: *const VNode) {
        todo!()
    }

    fn inactive(&self, c: super::vfs::UserCred, vp: *const VNode) {
        todo!()
    }

    fn bmap(&self, block_number: u32, bnp: (), vp: *const VNode) -> super::vfs::VNode {
        todo!()
    }

    fn strategy(&self, bp: (), vp: *const VNode) {
        todo!()
    }

    fn bread(&self, block_number: u32, vp: *const VNode) -> Arc<[u8]> {
        todo!()
    }
}

macro_rules! inode_enum_try_into {
    ($inode_type:ty, $inode_name:ident) => {
        impl<'a> TryInto<$inode_type> for Inode {
            type Error = ();

            fn try_into(self) -> Result<$inode_type, Self::Error> {
                match self {
                    Inode::$inode_name(inode) => {
                        return Ok(inode);
                    }
                    _ => {
                        return Err(());
                    }
                }
            }
        }
    };
}

inode_enum_try_into!(BasicFileInode, BasicFile);
inode_enum_try_into!(BasicDirectoryInode, BasicDirectory);

// TODO: can we remove the dependence on squahsfs??
#[repr(C)]
#[derive(Clone, Copy)]
struct InodeHeader {
    // squashfs: &'a Squashfs<'a>,
    file_type: InodeFileType,
    _reserved: [u16; 3],
    mtime: u32,
    inode_num: u32,
}

impl InodeHeader {
    fn from_bytes(bytes: &[u8]) -> Self {
        let file_type = u16::from_le_bytes(bytes[0..2].try_into().unwrap()).into();
        let mtime = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
        let inode_num = u32::from_le_bytes(bytes[12..16].try_into().unwrap());

        return Self {
            // squashfs,
            file_type,
            _reserved: [0; 3],
            mtime,
            inode_num,
        };
    }
}

impl Debug for InodeHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InodeHeader")
            .field("file_type", &self.file_type)
            .field("_reserved", &self._reserved)
            .field("mtime", &self.mtime)
            .field("inode_num", &self.inode_num)
            .finish()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct BasicDirectoryInode {
    header: InodeHeader,
    block_index: u32,  // 4
    link_count: u32,   // 8
    file_size: u16,    // 10
    block_offset: u16, // 12
    parent_inode: u32, // 16
}

impl BasicDirectoryInode {
    fn from_bytes(bytes: &[u8]) -> Self {
        let header = InodeHeader::from_bytes(bytes);
        let block_index = u32::from_le_bytes(bytes[16..20].try_into().unwrap());
        let link_count = u32::from_le_bytes(bytes[20..24].try_into().unwrap());
        let file_size = u16::from_le_bytes(bytes[24..26].try_into().unwrap());
        let block_offset = u16::from_le_bytes(bytes[26..28].try_into().unwrap());
        let parent_inode = u32::from_le_bytes(bytes[28..32].try_into().unwrap());

        return Self {
            header,
            block_index,
            link_count,
            file_size,
            block_offset,
            parent_inode,
        };
    }

    // #[allow(dead_code)]
    // fn entries(&self) -> Arc<[Inode]> {
    //     let mut entries: Vec<Inode> = Vec::new();

    //     let directory_table = &self
    //         .header
    //         .squashfs
    //         .get_decompressed_table(self.header.squashfs.directory_table, (true, None));

    //     let directory_table_header =
    //         DirectoryTableHeader::from_bytes(&directory_table[self.block_offset as usize..]);

    //     // TODO: cheap hack, fix it when I have more hours of sleep.
    //     let mut offset = self.block_offset as usize + core::mem::size_of::<DirectoryTableHeader>();

    //     for _ in 0..directory_table_header.entry_count as usize {
    //         let directory_table_entry = DirectoryTableEntry::from_bytes(&directory_table[offset..]);

    //         offset += 8 + directory_table_entry.name.len();

    //         let file_inode = self
    //             .header
    //             .squashfs
    //             .read_inode(directory_table_entry.offset as u32);

    //         entries.push(file_inode);
    //     }

    //     return Arc::from(entries);
    // }

    // fn find(&self, name: &str) -> Option<Inode<'a>> {
    //     let directory_table = &self
    //         .header
    //         .squashfs
    //         .get_decompressed_table(self.header.squashfs.directory_table, (true, None));

    //     let directory_table_header =
    //         DirectoryTableHeader::from_bytes(&directory_table[self.block_offset as usize..]);

    //     // TODO: cheap hack, fix it when I have more hours of sleep.
    //     let mut offset = self.block_offset as usize + core::mem::size_of::<DirectoryTableHeader>();
    //     InodeHeader
    //     for _ in 0..directory_table_header.entry_count as usize {
    //         let directory_table_entry = DirectoryTableEntry::from_bytes(&directory_table[offset..]);

    //         offset += 8 + directory_table_entry.name.len();

    //         if directory_table_entry.name == name {
    //             return Some(
    //                 self.header
    //                     .squashfs
    //                     .read_inode(directory_table_entry.offset as u32),
    //             );
    //         }
    //     }

    //     return None;
    // }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct BasicFileInode {
    header: InodeHeader,
    block_start: u32,  // 4
    frag_idx: u32,     // 8
    block_offset: u32, // 12
    file_size: u32,    // 16
    block_sizes: *const u32,
}

impl BasicFileInode {
    fn from_bytes(bytes: &[u8]) -> Self {
        let header = InodeHeader::from_bytes(bytes);
        let block_start = u32::from_le_bytes(bytes[16..20].try_into().unwrap());
        let frag_idx = u32::from_le_bytes(bytes[20..24].try_into().unwrap());
        let block_offset = u32::from_le_bytes(bytes[24..28].try_into().unwrap());
        let file_size = u32::from_le_bytes(bytes[28..32].try_into().unwrap());
        let block_sizes = bytes[32..].as_ptr() as *const u32;

        return Self {
            header,
            block_start,
            frag_idx,
            block_offset,
            file_size,
            block_sizes,
        };
    }
}

// impl<'a> VfsFile for BasicFileInode<'a> {
//     fn read(&self) -> Result<Arc<[u8]>, ()> {
//         // TODO: is this really how you're supposed to do this?
//         let mut block_data: Vec<u8> = Vec::with_capacity(self.file_size as usize);

//         let data_table: Vec<u8>;

//         let block_offset = if self.frag_idx == u32::MAX {
//             data_table = self.header.squashfs.get_decompressed_table(
//                 self.header.squashfs.data_table,
//                 (
//                     false,
//                     Some(
//                         !self
//                             .header
//                             .squashfs
//                             .superblock
//                             .features()
//                             .uncompressed_data_blocks,
//                     ),
//                 ),
//             );

//             self.block_offset as usize
//         } else {
//             // Tail end packing
//             let fragment_table = self.header.squashfs.get_decompressed_table(
//                 self.header.squashfs.fragment_table.unwrap(),
//                 (
//                     false,
//                     Some(false), // Some(
//                                  //     !self
//                                  //         .header
//                                  //         .squashfs
//                                  //         .superblock
//                                  //         .features()
//                                  //         .uncompressed_fragments,
//                                  // ),
//                 ),
//             );

//             let fragment_pointer = (self.header.squashfs.start as u64
//                 + u64::from_le_bytes(
//                     fragment_table[self.frag_idx as usize..(self.frag_idx + 8) as usize]
//                         .try_into()
//                         .unwrap(),
//                 )) as *mut u8;

//             // build array since fragment_pointer is not guaranteed to be 0x02 aligned
//             // We add two since fragment_pointer points to the beginning of the fragment block,
//             // Which is a metadata block, and we get the size, but that excludes the two header bytes,
//             // And since we are building the array due to unaligned pointer shenanigans we need to
//             // include the header bytes otherwise we are short by two bytes
//             let fragment_block_size = unsafe {
//                 u16::from_le(core::ptr::read_unaligned(fragment_pointer as *mut u16)) & 0x7FFF
//             } + 2;

//             let mut fragment_block_raw = Vec::new();
//             for i in 0..fragment_block_size as usize {
//                 fragment_block_raw
//                     .push(unsafe { core::ptr::read_unaligned(fragment_pointer.add(i)) })
//             }

//             let fragment_block = self
//                 .header
//                 .squashfs
//                 .get_decompressed_table(&fragment_block_raw, (true, None));

//             let fragment_start = u64::from_le_bytes(fragment_block[0..8].try_into().unwrap());
//             let fragment_size = u32::from_le_bytes(fragment_block[8..12].try_into().unwrap());
//             let fragment_compressed = fragment_size & 1 << 24 == 0;
//             let fragment_size = fragment_size & 0xFEFFFFFF;

//             let data_table_raw = unsafe {
//                 core::slice::from_raw_parts(
//                     (self.header.squashfs.start as u64 + fragment_start) as *mut u8,
//                     fragment_size as usize,
//                 )
//                 .to_vec()
//             };

//             data_table = self
//                 .header
//                 .squashfs
//                 .get_decompressed_table(&data_table_raw, (false, Some(fragment_compressed)));

//             self.block_offset as usize
//         };

//         block_data.extend(&data_table[block_offset..(block_offset + self.file_size as usize)]);

//         return Ok(Arc::from(block_data));
//     }
// }

#[repr(C)]
#[derive(Debug)]
struct DirectoryTableHeader {
    entry_count: u32,
    start: u32,
    inode_num: u32,
}

impl DirectoryTableHeader {
    fn from_bytes(bytes: &[u8]) -> Self {
        // count is off by 1 entry
        let entry_count = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) + 1;
        let start = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        let inode_num = u32::from_le_bytes(bytes[8..12].try_into().unwrap());

        return Self {
            entry_count,
            start,
            inode_num,
        };
    }
}

#[repr(C)]
#[derive(Debug)]
struct DirectoryTableEntry<'a> {
    offset: u16,
    inode_offset: i16,
    inode_type: InodeFileType,
    name_size: u16,
    name: &'a str, // the file name length is name_size + 1 bytes
}

impl<'a> DirectoryTableEntry<'a> {
    fn from_bytes(bytes: &'a [u8]) -> Self {
        let offset = u16::from_le_bytes(bytes[0..2].try_into().unwrap());
        let inode_offset = i16::from_le_bytes(bytes[2..4].try_into().unwrap());
        let inode_type = u16::from_le_bytes(bytes[4..6].try_into().unwrap()).into();
        let name_size = u16::from_le_bytes(bytes[6..8].try_into().unwrap());
        let name = core::str::from_utf8(&bytes[8..((name_size as usize) + 1) + 8])
            .expect("Failed to make DirectoryHeader name");

        return Self {
            offset,
            inode_offset,
            inode_type,
            name_size,
            name,
        };
    }
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

impl From<u16> for InodeFileType {
    fn from(value: u16) -> Self {
        match value {
            1 => Self::BasicDirectory,
            2 => Self::BasicFile,
            3 => Self::BasicSymlink,
            4 => Self::BasicBlockDevice,
            5 => Self::BasicCharDevice,
            6 => Self::BasicPipe,
            7 => Self::BasicSocked,
            8 => Self::ExtendedDirectory,
            9 => Self::ExtendedFile,
            10 => Self::ExtendedSymlink,
            11 => Self::ExtendedBlockDevice,
            12 => Self::ExtendedPipe,
            13 => Self::ExtendedSocked,
            _ => panic!("Unexpected Inode file type {value}!"),
        }
    }
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SquashfsCompressionType {
    Gzip = 1,
    Lzma = 2,
    Lzo = 3,
    Xz = 4,
    Lz4 = 5,
    Zstd = 6,
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

#[allow(dead_code)]
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

impl From<u16> for SquashfsCompressionType {
    fn from(value: u16) -> Self {
        match value {
            1 => Self::Gzip,
            2 => Self::Lzma,
            3 => Self::Lzo,
            4 => Self::Xz,
            5 => Self::Lz4,
            6 => Self::Zstd,
            _ => panic!("Unexpected Squashfs compression type!"),
        }
    }
}

#[repr(C, packed)]
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
