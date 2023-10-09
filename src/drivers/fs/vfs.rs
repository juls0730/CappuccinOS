// use alloc::{boxed::Box, string::String, vec::Vec};

// use crate::drivers::storage::drive::BlockDevice;

// // Define a custom error enum for file-related errors
// enum VfsError {
//     NotFound,
//     FileExists,
// }

// trait VfsDevice {
//     fn mount(&self, mount_point: &str) -> Result<(), ()>;

//     fn open(&self, path: &str) -> Result<Box<dyn VfsFile>, VfsError>;

//     fn create(&self, path: &str) -> Result<Box<dyn VfsFile>, VfsError>;

//     fn remove(&self, path: &str) -> Result<(), VfsError>;

//     fn list_directory(&self, path: &str) -> Result<Vec<String>, VfsError>;
// }

// trait VfsFileSystem {
//     // Mount the file system at the specified mount point
//     fn mount(&mut self, mount_point: &str) -> Result<(), VfsError>;

//     // Open a file by its path and return a file handle
//     fn open_file(&self, path: &str) -> Result<Box<dyn VfsFile>, VfsError>;

//     // Create a new file at the specified path
//     fn create_file(&self, path: &str) -> Result<Box<dyn VfsFile>, VfsError>;

//     // Remove a file or directory at the specified path
//     fn remove(&self, path: &str) -> Result<(), VfsError>;

//     // List the contents of a directory
//     fn list_directory(&self, path: &str) -> Result<Vec<String>, VfsError>;
// }

// struct VfsBlockDevice {
//     inner: Box<dyn BlockDevice>,
// }

// impl VfsBlockDevice {
//     pub fn new(inner: Box<dyn BlockDevice>) -> Self {
//         Self { inner }
//     }
// }

// impl BlockDevice for VfsBlockDevice {
//     fn read(&self, sector: u64, sector_count: usize) -> Result<alloc::sync::Arc<[u8]>, ()> {
//         return self.inner.read(sector, sector_count);
//     }

//     fn sector_count(&self) -> u64 {
//         return self.inner.sector_count();
//     }

//     fn write(&self, sector: u64, data: &[u8]) -> Result<(), ()> {
//         return self.inner.write(sector, data);
//     }
// }

// // Implement the VfsDevice trait for VfsBlockDevice with the required VFS-specific methods
// impl VfsDevice for VfsBlockDevice {
//     fn mount(&self, mount_point: &str) -> Result<(), VfsError> {
//         // Implement the mounting logic here
//         // You can use the inner BlockDevice for low-level access
//         // and provide VFS-specific functionality
//         // ...
//     }

//     // ... implement other VfsDevice methods ...
// }
