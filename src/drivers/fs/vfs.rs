use alloc::{
    boxed::Box,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};

use crate::libs::mutex::Mutex;

pub trait VfsFileSystem {
    fn open(&self, path: &str) -> Result<Box<dyn VfsFile + '_>, ()>;
    fn read_dir(&self, path: &str) -> Result<Box<dyn VfsDirectory>, ()>;
}

pub trait VfsFile {
    fn read(&self) -> Result<Arc<[u8]>, ()>;
}

pub trait VfsDirectory {
    fn list_files(&self) -> Result<Arc<[Box<dyn VfsFile>]>, ()>;
}

pub static VFS_INSTANCES: Mutex<Vec<Vfs>> = Mutex::new(Vec::new());

pub struct Vfs {
    identifier: String,
    file_system: Box<dyn VfsFileSystem>,
}

impl Vfs {
    pub fn new(file_system: Box<dyn VfsFileSystem>, identifier: &str) -> Self {
        return Self {
            identifier: identifier.to_string(),
            file_system,
        };
    }

    pub fn open(&self, path: &str) -> Result<Box<dyn VfsFile + '_>, ()> {
        return self.file_system.open(path);
    }

    pub fn read_dir(&self, path: &str) -> Result<Box<dyn VfsDirectory>, ()> {
        return self.file_system.read_dir(path);
    }
}

pub fn init() {
    // TODO: Deduce which storage medium(s) we're using
    crate::drivers::storage::ide::init();
}
