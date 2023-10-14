use alloc::{boxed::Box, sync::Arc, vec::Vec};

use crate::libs::mutex::Mutex;

pub trait VFSFileSystem {
    fn open(&self, path: &str) -> Result<Box<dyn VFSFile + '_>, ()>;
    fn read_dir(&self, path: &str) -> Result<Box<dyn VFSDirectory>, ()>;
}

pub trait VFSFile {
    fn read(&self) -> Result<Arc<[u8]>, ()>;
}

pub trait VFSDirectory {
    fn list_files(&self) -> Result<Arc<[Box<dyn VFSFile>]>, ()>;
}

pub static VFS_INSTANCES: Mutex<Vec<VFS>> = Mutex::new(Vec::new());

pub struct VFS {
    file_system: Box<dyn VFSFileSystem>,
}

impl VFS {
    pub fn new(file_system: Box<dyn VFSFileSystem>) -> Self {
        return Self { file_system };
    }

    pub fn open(&self, path: &str) -> Result<Box<dyn VFSFile + '_>, ()> {
        return self.file_system.open(path);
    }

    pub fn read_dir(&self, path: &str) -> Result<Box<dyn VFSDirectory>, ()> {
        return self.file_system.read_dir(path);
    }
    // Add more VFS methods as needed
}

pub fn init() {
    // TODO: Deduce which storage medium(s) we're using
    crate::drivers::storage::ide::init();
}

// //? Please, shield your eyes and hide your children

// // This is... really bad to say the least, I think you can make it way better by using traits but I'm not sure

// use alloc::{sync::Arc, vec::Vec};

// use crate::libs::mutex::Mutex;

// pub struct VFSPartition {
//     start_sector: usize,
//     sectors: usize,
// }

// pub struct VFSDrive {
//     sectors: usize,
//     read: fn(sector: u64, sector_count: usize) -> Result<Arc<[u8]>, ()>,
//     write: fn(sector: u64, data: &[u8]) -> Result<(), ()>,
//     partitions: Arc<[VFSPartition]>,
// }

// pub struct VFSFileSystem<'a> {
//     name: &'a str,
//     mount_point: &'a str,
//     open: Option<fn(VFSNode)>,
//     read: Option<fn(VFSNode, usize, usize) -> Result<Arc<[u8]>, ()>>,
//     write: Option<fn(VFSNode, Arc<[u8]>, usize, usize) -> Result<(), ()>>,
// }

// pub struct VFSNode<'a> {
//     filesystem: *const VFSFileSystem<'a>,
//     size: usize,
//     id: usize,
//     path: &'a str,

//     next: Option<*const VFSNode<'a>>,
// }

// static ROOT_FS: VFSFileSystem = VFSFileSystem {
//     name: "rootfs",
//     mount_point: "/",
//     open: None,
//     read: None,
//     write: None,
// };

// static ROOT_NODE: Mutex<VFSNode> = Mutex::new(VFSNode {
//     filesystem: core::ptr::null(),
//     size: 0,
//     id: 0,
//     path: "",
//     next: None,
// });

// pub static VFS_INSTANCES: Mutex<VFS> = Mutex::new(VFS::new());

// pub struct VFS {
//     drives: Vec<VFSDrive>,
// }

// impl VFS {
//     pub const fn new() -> Self {
//         return Self { drives: Vec::new() };
//     }

//     pub fn add_drive(&mut self, drive: VFSDrive) {
//         self.drives.push(drive);

//         let mut drive_node = VFSNode {
//             filesystem: core::ptr::null(),
//             size: 0,
//             id: 0,
//             path: "",
//             next: None,
//         };

//         drive_node.filesystem = &ROOT_FS;
//     }
// }

// pub fn init() {
//     ROOT_NODE.lock().write().filesystem = &ROOT_FS;

//     crate::println!("VFS: Registered VFS on {}", ROOT_FS.mount_point);
// }
