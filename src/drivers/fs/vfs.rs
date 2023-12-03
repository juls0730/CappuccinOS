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
