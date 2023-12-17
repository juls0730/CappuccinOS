use alloc::{
    boxed::Box,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};

use crate::drivers::storage::drive::{BlockDevice, GPTPartitionEntry};

use super::vfs::{VfsDirectory, VfsFile, VfsFileSystem};

// The first Cluster (perhaps 0xF0FFFF0F) is the FAT ID
// The second cluster stores the end-of-cluster-chain marker
// The third entry and further holds the directory table
//
// Fat Clusters are either one of these types:
//
// 0x0FFFFFF8 : End Of cluster Chain
// 0x0FFFFFF7 : Bad Cluster
// 0x00000001 - 0x0FFFFFEF : In use Cluster
// 0x00000000 : Free Cluster

// End Of Chain
const EOC: u32 = 0x0FFFFFF8;

#[derive(Clone, Copy, Debug)]
enum FatType {
    Fat12,
    Fat16,
    Fat32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct BIOSParameterBlock {
    _jmp_instruction: [u8; 3],     // EB 58 90
    pub oem_identifier: [u8; 8],   // MTOO4043 (hey, mtools)
    pub bytes_per_sector: u16,     // 00 02 (little endian so 512)
    pub sectors_per_cluster: u8,   // 01
    pub reserved_sectors: u16,     // 20 00 (32)
    pub fat_count: u8,             // 02
    pub root_directory_count: u16, // 00 00 (what)
    pub total_sectors: u16,        // equal to zero when sector count is more than 65535
    pub media_descriptor_type: u8, // F0
    pub sectors_per_fat: u16,      // Fat12/Fat16 only
    pub sectors_per_track: u16,    // 3F 00 (63)
    pub head_count: u16,           // 10 00 (16)
    pub hidden_sectors: u32,       // 00 00 00 00
    pub large_sector_count: u32,   // 00 F8 01 00 (129024)
    // pub ebpb: [u8; 54],
    // ---------------------------------
    // - Extended BIOS Parameter Block -
    // ---------------------------------
    pub sectors_per_fat_ext: u32,          // E1 03 00 00 (993, wtf)
    pub flags: [u8; 2],                    // 00 00
    pub fat_version: u16,                  // 00 00
    pub root_dir_cluster: u32,             // 2C 00 00 00 (2)
    pub fsinfo_sector: u16,                // 01 00 (1)
    pub backup_bootsector: u16,            // 06 00 (6)
    _reserved: [u8; 12],                   // all zero
    pub drive_number: u8,                  // 00
    _reserved2: u8,                        // 00
    pub signature: u8,                     // either 0x28 of 0x29: 29
    pub volume_id: u32,                    // Varies
    pub volume_label: [u8; 11],            // "NO NAME    "
    pub system_identifier_string: [u8; 8], // Always "FAT32   " but never trust the contents of this string (for some reason)
    _boot_code: [u8; 420],                 // ~~code~~
    pub bootable_signature: u16,           // 0xAA55
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct FSInfo {
    pub lead_signature: u32,
    _reserved: [u8; 480],
    pub mid_signature: u32,
    pub last_known_free_cluster: u32,
    pub look_for_free_clusters: u32,
    _reserved2: [u8; 12],
    pub trail_signature: u32,
}

impl FSInfo {
    pub fn from_bytes(bytes: Arc<[u8]>) -> Self {
        let lead_signature = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let _reserved = bytes[4..484].try_into().unwrap();
        let mid_signature = u32::from_le_bytes(bytes[484..488].try_into().unwrap());
        let last_known_free_cluster = u32::from_le_bytes(bytes[488..492].try_into().unwrap());
        let look_for_free_clusters = u32::from_le_bytes(bytes[492..496].try_into().unwrap());
        let _reserved2 = bytes[496..508].try_into().unwrap();
        let trail_signature = u32::from_le_bytes(bytes[508..].try_into().unwrap());

        return Self {
            lead_signature,
            _reserved,
            mid_signature,
            last_known_free_cluster,
            look_for_free_clusters,
            _reserved2,
            trail_signature,
        };
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq)]
enum FileEntryAttributes {
    ReadOnly = 0x01,
    Hidden = 0x02,
    System = 0x04,
    VolumeId = 0x08,
    Directory = 0x10,
    Archive = 0x20, // basically any file
    LongFileName = 0x0F,
}

#[repr(packed)]
#[derive(Debug)]
struct LongFileName {
    entry_order: u8,
    first_characters: [u16; 5],
    attribute: u8,       // always 0x0F
    long_entry_type: u8, // zero for name entries
    checksum: u8,
    second_characters: [u16; 6],
    _always_zero: [u8; 2],
    final_characters: [u16; 2],
}

#[repr(packed)]
#[derive(Debug)]
pub struct FileEntry {
    file_name: [u8; 8],
    extension: [u8; 3],
    attributes: u8,
    _reserved: u8,
    creation_tenths: u8,
    creation_time: u16,
    creation_date: u16,
    accessed_date: u16,
    high_first_cluster_number: u16, // The high 16 bits of this entry's first cluster number. For FAT 12 and FAT 16 this is always zero.
    modified_time: u16,
    modified_date: u16,
    low_first_cluster_number: u16,
    file_size: u32,
}

pub struct FatFs<'a> {
    // Block device Info
    drive: &'a dyn BlockDevice,
    partition: GPTPartitionEntry,
    // FAT info
    fs_info: FSInfo,
    fat: Option<Arc<[u32]>>,
    bpb: BIOSParameterBlock,
    fat_start: u64,
    fat_type: FatType,
    cluster_size: usize,
    sectors_per_fat: usize,
}

impl<'a> FatFs<'a> {
    pub fn new(drive: &'a dyn BlockDevice, partition: GPTPartitionEntry) -> Result<Self, ()> {
        let bpb_bytes = drive
            .read(partition.start_sector, 1)
            .expect("Failed to read FAT32 BIOS Parameter Block!");

        let bpb = unsafe { *(bpb_bytes.clone().as_ptr() as *const BIOSParameterBlock) };

        let system_identifier = core::str::from_utf8(&bpb.system_identifier_string);

        if system_identifier.is_err() {
            return Err(());
        }

        // We're trusting it
        if let Ok(system_identifier_string) = system_identifier {
            if !system_identifier_string.contains("FAT32") {
                return Err(());
            }
        }

        let fsinfo_bytes = drive
            .read(partition.start_sector + bpb.fsinfo_sector as u64, 1)
            .expect("Failed to read FSInfo sector!");

        let fs_info = FSInfo::from_bytes(fsinfo_bytes);

        let fat_start = partition.start_sector + bpb.reserved_sectors as u64;

        let bytes_per_fat = 512 * bpb.sectors_per_fat_ext as usize;

        let mut fat: Option<Arc<[u32]>> = None;

        if crate::KERNEL_FEATURES.fat_in_mem {
            let mut fat_vec: Vec<u32> = Vec::with_capacity(bytes_per_fat / 4);

            for i in 0..(bpb.sectors_per_fat_ext as usize) {
                let sector = drive
                    .read(fat_start + i as u64, 1)
                    .expect("Failed to read FAT");
                for j in 0..(512 / 4) {
                    fat_vec.push(u32::from_le_bytes(
                        sector[j * 4..(j * 4 + 4)].try_into().unwrap(),
                    ))
                }
            }

            fat = Some(Arc::from(fat_vec));
        } else {
            crate::log_info!(
                "\033[33mWARNING\033[0m: FAT is not being stored in memory, this feature is experimental and file reads are expected to be slower."
            )
        }

        let (total_sectors, fat_size) = if bpb.total_sectors == 0 {
            (bpb.large_sector_count, bpb.sectors_per_fat_ext)
        } else {
            (bpb.total_sectors as u32, bpb.sectors_per_fat as u32)
        };

        let root_dir_sectors =
            ((bpb.root_directory_count * 32) + (bpb.bytes_per_sector - 1)) / bpb.bytes_per_sector;
        let total_data_sectors = total_sectors
            - (bpb.reserved_sectors as u32
                + (bpb.fat_count as u32 * fat_size)
                + root_dir_sectors as u32);

        let total_clusters = total_data_sectors / bpb.sectors_per_cluster as u32;

        let fat_type = if total_clusters < 4085 {
            FatType::Fat12
        } else if total_clusters < 65525 {
            FatType::Fat16
        } else {
            FatType::Fat32
        };

        crate::println!("Found {fat_type:?} FS");

        let sectors_per_fat = match fat_type {
            FatType::Fat32 => bpb.sectors_per_fat_ext as usize,
            _ => bpb.sectors_per_fat as usize,
        };

        let cluster_size = bpb.sectors_per_cluster as usize * 512;

        return Ok(Self {
            drive,
            partition,
            fs_info,
            fat,
            bpb,
            fat_start,
            fat_type,
            cluster_size,
            sectors_per_fat,
        });
    }

    fn find_entry_in_directory(&self, cluster: usize, name: &str) -> Result<FileEntry, ()> {
        let mut i: usize = 0;
        // Long file name is stored outsize because long filename and the real entry on separate entries
        let mut long_filename: Vec<LongFileName> = Vec::new();
        let mut long_filename_string: Option<String> = None;

        let data_sector = self.read_cluster(cluster)?;

        loop {
            let bytes: [u8; core::mem::size_of::<FileEntry>()] =
                data_sector[(i * 32)..((i + 1) * 32)].try_into().unwrap();
            let first_byte = bytes[0];

            let file_entry: FileEntry;

            i += 1;

            // Step 1
            if first_byte == 0x00 {
                break; // End of directory listing
            }

            // Step 2
            if first_byte == 0xE5 {
                continue; // Directory is unused, ignore it
            } else if bytes[11] == FileEntryAttributes::LongFileName as u8 {
                // Entry is LFN (step 3)
                // read long filename (step 4)
                let long_filename_part: LongFileName;

                unsafe {
                    long_filename_part = core::mem::transmute(bytes);
                }
                long_filename.push(long_filename_part);
                continue;
            } else {
                // step 5
                unsafe {
                    file_entry = core::mem::transmute(bytes);
                }

                // step 6
                if !long_filename.is_empty() {
                    // Make fileEntry with LFN (step 7)
                    let mut string: Vec<u16> = Vec::with_capacity(long_filename.len() * 13);

                    for i in 0..long_filename.len() {
                        let i = (long_filename.len() - 1) - i;
                        let long_filename = &long_filename[i];

                        let mut character_bytes = Vec::new();
                        let characters = long_filename.first_characters;

                        character_bytes.extend_from_slice(&characters);
                        let characters = long_filename.second_characters;

                        character_bytes.extend_from_slice(&characters);
                        let characters = long_filename.final_characters;

                        character_bytes.extend_from_slice(&characters);

                        // remove 0x0000 characters and 0xFFFF characters
                        character_bytes.retain(|&x| x != 0xFFFF && x != 0x0000);

                        for &le_character in character_bytes.iter() {
                            // Convert little-endian u16 to native-endian u16
                            let native_endian_value = u16::from_le(le_character);
                            string.push(native_endian_value);
                        }
                    }
                    long_filename_string = Some(String::from_utf16(&string).unwrap());
                    long_filename.clear();
                }
            }

            if name.replacen('.', "", 1).len() <= 11 {
                let search_parts: Vec<&str> = name.split('.').collect();

                let filename = core::str::from_utf8(&file_entry.file_name).unwrap();
                let extension = core::str::from_utf8(&file_entry.extension).unwrap();

                if (search_parts.len() == 1
                    && !filename.contains(&search_parts[0].to_ascii_uppercase()))
                    || (search_parts.len() > 1
                        && (!filename.contains(&search_parts[0].to_ascii_uppercase())
                            || !extension.contains(&search_parts[1].to_ascii_uppercase())))
                {
                    continue;
                }

                return Ok(file_entry);
            } else {
                // Long file name
                if long_filename_string != Some(name.to_string()) {
                    continue;
                }

                return Ok(file_entry);
            }
        }

        return Err(());
    }

    pub fn read_cluster(&self, cluster: usize) -> Result<Arc<[u8]>, ()> {
        return self.drive.read(
            self.partition.start_sector + self.cluster_to_sector(cluster) as u64,
            self.bpb.sectors_per_cluster as usize,
        );
    }

    fn cluster_to_sector(&self, cluster: usize) -> usize {
        let fat_size = self.sectors_per_fat;
        let root_dir_sectors = ((self.bpb.root_directory_count * 32)
            + (self.bpb.bytes_per_sector - 1))
            / self.bpb.bytes_per_sector;

        let first_data_sector = self.bpb.reserved_sectors as usize
            + (self.bpb.fat_count as usize * fat_size)
            + root_dir_sectors as usize;

        return ((cluster - 2) as isize * self.bpb.sectors_per_cluster as isize) as usize
            + first_data_sector;
    }

    fn get_next_cluster(&self, cluster: usize) -> u32 {
        if crate::KERNEL_FEATURES.fat_in_mem {
            return match self.fat_type {
                FatType::Fat12 => {
                    todo!();
                }
                FatType::Fat16 => {
                    todo!();
                }
                FatType::Fat32 => self.fat.as_ref().unwrap()[cluster] & 0x0FFFFFFF,
            };
        } else {
            let fat_entry_size = match self.fat_type {
                FatType::Fat12 => 1, // 12 bits per entry
                FatType::Fat16 => 2, // 16 bits per entry
                FatType::Fat32 => 4, // "32" bits per entry
            };
            let entry_offset = cluster * fat_entry_size;
            let entry_offset_in_sector = entry_offset % 512;

            let sector_data = self
                .drive
                .read(self.fat_start + entry_offset as u64 / 512, 1)
                .expect("Failed to read from FAT!");

            match self.fat_type {
                FatType::Fat12 => {
                    todo!();
                }
                FatType::Fat16 => {
                    todo!();
                }
                FatType::Fat32 => {
                    let cluster_entry_bytes: [u8; 4] = sector_data
                        [entry_offset_in_sector..=entry_offset_in_sector + 3]
                        .try_into()
                        .unwrap();
                    return u32::from_le_bytes(cluster_entry_bytes) & 0x0FFFFFFF;
                }
            }
        }
    }
}

impl<'a> VfsFileSystem for FatFs<'a> {
    fn open(&self, path: &str) -> Result<Box<dyn VfsFile + '_>, ()> {
        let path_componenets: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        let mut current_cluster = self.bpb.root_dir_cluster as usize;

        for path in path_componenets {
            let file_entry: FileEntry = self.find_entry_in_directory(current_cluster, path)?;

            if file_entry.attributes == FileEntryAttributes::Directory as u8 {
                current_cluster = (((file_entry.high_first_cluster_number as u32) << 16)
                    | file_entry.low_first_cluster_number as u32)
                    as usize;
            } else {
                return Ok(Box::new(FatFile {
                    fat_fs: self,
                    file_entry,
                }));
            }
        }

        return Err(());
    }

    fn read_dir(&self, path: &str) -> Result<Box<dyn VfsDirectory>, ()> {
        unimplemented!();
    }
}

struct FatFile<'a> {
    fat_fs: &'a FatFs<'a>,
    file_entry: FileEntry,
}

impl<'a> VfsFile for FatFile<'a> {
    fn read(&self) -> Result<Arc<[u8]>, ()> {
        let mut file: Vec<u8> = Vec::with_capacity(self.file_entry.file_size as usize);
        let mut file_ptr_index = 0;

        let mut cluster = ((self.file_entry.high_first_cluster_number as u32) << 16)
            | self.file_entry.low_first_cluster_number as u32;
        let cluster_size = self.fat_fs.cluster_size;

        let mut copied_bytes = 0;

        loop {
            let cluster_data = self.fat_fs.read_cluster(cluster as usize)?;

            let remaining = self.file_entry.file_size as usize - copied_bytes;
            let to_copy = if remaining > cluster_size {
                cluster_size
            } else {
                remaining
            };

            unsafe {
                core::ptr::copy_nonoverlapping(
                    cluster_data.as_ptr(),
                    file.as_mut_ptr().add(file_ptr_index),
                    to_copy,
                );

                file.set_len(file.len() + to_copy);
            }

            file_ptr_index += cluster_size;

            copied_bytes += to_copy;

            cluster = self.fat_fs.get_next_cluster(cluster as usize);

            if cluster >= EOC {
                break;
            }
        }

        return Ok(Arc::from(file));
    }
}

struct FatDirectory<'a> {
    fat_fs: &'a FatFs<'a>,
    directory_cluster: usize,
}

impl<'a> VfsDirectory for FatDirectory<'a> {
    fn list_files(&self) -> Result<Arc<[Box<dyn VfsFile>]>, ()> {
        unimplemented!();
    }
}
