use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};

use crate::drivers::storage::drive::{BlockDevice, GPTPartitionEntry};

enum FatType {
    Fat12,
    Fat16,
    Fat32,
}

#[repr(C, packed)]
#[derive(Debug)]
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
}

impl BIOSParameterBlock {
    pub fn from_bytes(bytes: Arc<[u8]>) -> Self {
        // Parse the individual fields from the byte array
        let jmp_instruction = bytes[0..3].try_into().unwrap();
        let oem_identifier = bytes[3..11].try_into().unwrap();
        let bytes_per_sector = u16::from_le_bytes(bytes[11..13].try_into().unwrap());
        let sectors_per_cluster = bytes[13];
        let reserved_sectors = u16::from_le_bytes(bytes[14..16].try_into().unwrap());
        let fat_count = bytes[16];
        let root_directory_count = u16::from_le_bytes(bytes[17..19].try_into().unwrap());
        let total_sectors = u16::from_le_bytes(bytes[19..21].try_into().unwrap());
        let media_descriptor_type = bytes[21];
        let sectors_per_fat = u16::from_le_bytes(bytes[22..24].try_into().unwrap());
        let sectors_per_track = u16::from_le_bytes(bytes[24..26].try_into().unwrap());
        let head_count = u16::from_le_bytes(bytes[26..28].try_into().unwrap());
        let hidden_sectors = u32::from_le_bytes(bytes[28..32].try_into().unwrap());
        let large_sector_count = u32::from_le_bytes(bytes[32..36].try_into().unwrap());

        return Self {
            _jmp_instruction: jmp_instruction,
            oem_identifier,
            bytes_per_sector,
            sectors_per_cluster,
            reserved_sectors,
            fat_count,
            root_directory_count,
            total_sectors,
            media_descriptor_type,
            sectors_per_fat,
            sectors_per_track,
            head_count,
            hidden_sectors,
            large_sector_count,
        };
    }
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct ExtendedBIOSParameterBlock {
    pub sectors_per_fat: u32,           // E1 03 00 00 (993, wtf)
    pub flags: [u8; 2],                 // 00 00
    pub fat_version: u16,               // 00 00
    pub root_dir_cluster: u32,          // 2C 00 00 00 (2)
    pub fsinfo_sector: u16,             // 01 00 (1)
    pub backup_bootsector: u16,         // 06 00 (6)
    _reserved: [u8; 12],                // all zero
    pub drive_number: u8,               // 00
    _reserved2: u8,                     // 00
    pub signature: u8,                  // either 0x28 of 0x29: 29
    pub volume_id: u32,                 // Varies
    pub volume_label: [u8; 11],         // "NO NAME    "
    _system_identifier_string: [u8; 8], // Always "FAT32   " but never trust the contents of this string (for some reason)
    _boot_code: [u8; 420],              // ~~code~~
    _bootable_signature: u16,           // 0xAA55
}

impl ExtendedBIOSParameterBlock {
    pub fn from_bytes(bytes: Arc<[u8]>) -> Self {
        // Parse the individual fields from the byte array
        let sectors_per_fat = u32::from_le_bytes(bytes[36..40].try_into().unwrap());
        let flags = bytes[40..42].try_into().unwrap();
        let fat_version = u16::from_le_bytes(bytes[42..44].try_into().unwrap());
        let root_dir_cluster = u32::from_le_bytes(bytes[44..48].try_into().unwrap());
        let fsinfo_sector = u16::from_le_bytes(bytes[48..50].try_into().unwrap());
        let backup_bootsector = u16::from_le_bytes(bytes[50..52].try_into().unwrap());
        let _reserved = bytes[52..64].try_into().unwrap();
        let drive_number = bytes[64];
        let _reserved2 = bytes[65];
        let signature = bytes[66];
        let volume_id = u32::from_le_bytes(bytes[67..71].try_into().unwrap());
        let volume_label = bytes[71..82].try_into().unwrap();
        let _system_identifier_string = bytes[82..90].try_into().unwrap();
        let _boot_code = bytes[90..510].try_into().unwrap();
        let _bootable_signature = u16::from_le_bytes(bytes[510..].try_into().unwrap());

        return Self {
            sectors_per_fat,
            flags,
            fat_version,
            root_dir_cluster,
            fsinfo_sector,
            backup_bootsector,
            _reserved,
            drive_number,
            _reserved2,
            signature,
            volume_id,
            volume_label,
            _system_identifier_string,
            _boot_code,
            _bootable_signature,
        };
    }
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
struct FileEntry /*<'a>*/ {
    filename: [u8; 8],
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
    // long_file_name: Option<&'a str>,
}

pub struct FATFS<'a> {
    drive: Box<&'a dyn BlockDevice>,
    bpb: BIOSParameterBlock,
    ebpb: ExtendedBIOSParameterBlock,
    fs_info: FSInfo,
    partition: GPTPartitionEntry,
}

impl<'a> FATFS<'a> {
    pub fn new(drive: &'a dyn BlockDevice, partition: GPTPartitionEntry) -> Self {
        let bpb_bytes = drive
            .read(partition.start_sector, 1)
            .expect("Failed to read FAT32 BIOS Parameter Block!");

        let bpb = BIOSParameterBlock::from_bytes(bpb_bytes.clone());
        let ebpb = ExtendedBIOSParameterBlock::from_bytes(bpb_bytes);

        let fsinfo_bytes = drive
            .read(partition.start_sector + ebpb.fsinfo_sector as u64, 1)
            .expect("Failed to read FSInfo sector!");

        let fs_info = FSInfo::from_bytes(fsinfo_bytes);

        return Self {
            drive: Box::new(drive),
            bpb,
            ebpb,
            fs_info,
            partition,
        };
    }

    pub fn test(&self) {
        let bpb = &self.bpb;
        let ebpb = &self.ebpb;

        let total_sectors = bpb.large_sector_count;
        let fat_size = ebpb.sectors_per_fat;
        let root_dir_sectors =
            ((bpb.root_directory_count * 32) + (bpb.bytes_per_sector - 1)) / bpb.bytes_per_sector;
        let first_data_sector = bpb.reserved_sectors as u32
            + (bpb.fat_count as u32 * fat_size)
            + root_dir_sectors as u32;
        let first_fat_sector = bpb.reserved_sectors;
        let total_data_sectors = total_sectors
            - (bpb.reserved_sectors as u32
                + (bpb.fat_count as u32 * fat_size)
                + root_dir_sectors as u32);
        let total_clusters = total_data_sectors / bpb.sectors_per_cluster as u32;
        let first_root_dir = first_data_sector - root_dir_sectors as u32;
        let root_cluster = ebpb.root_dir_cluster;
        let cluster = 0;
        let first_sector_of_cluster = self.cluster_to_sector(cluster);

        // TODO
        crate::println!(
            "First Data sector offset: {:#X}, First FAT Sector offset: {:#X}, First sector of cluster offset: {:#X}",
            (self.partition.start_sector + first_data_sector as u64) * 512,
            (self.partition.start_sector + first_fat_sector as u64) * 512,
            (self.partition.start_sector + first_sector_of_cluster as u64) * 512
        );

        let fat = self
            .drive
            .read(self.partition.start_sector + first_fat_sector as u64, 1)
            .expect("Failed to read FAT!");

        let data_sector = self
            .drive
            .read(
                self.partition.start_sector + first_data_sector as u64 + 6,
                1,
            )
            .expect("Failed to read FATFS data sector!");

        // Loop over entries
        let mut i: usize = 0;
        // Long file name is stored outsize because long filename and the real entry on separate entries
        let mut long_filename: Vec<LongFileName> = Vec::new();
        let search = "CappuccinOS.elf";

        let mut long_filename_string: Option<String> = None;

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
                // read long filename somehow (step 4)
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

                    crate::println!("Long file name: {:?}", long_filename_string);
                    long_filename.clear();
                }
            }

            if search.len() < 11 {
                let search_parts: Vec<&str> = search.split(".").collect();

                let filename = core::str::from_utf8(&file_entry.filename).unwrap();
                let extension = core::str::from_utf8(&file_entry.extension).unwrap();

                if !filename.contains(&search_parts[0].to_ascii_uppercase())
                    || !extension.contains(&search_parts[1].to_ascii_uppercase())
                {
                    continue;
                }

                let file_cluster = u32::from_le(file_entry.low_first_cluster_number as u32);
                let file_sector = self.partition.start_sector as usize
                    + self.cluster_to_sector(file_cluster as usize);

                // crate::println!("start: {} data: {first_data_sector} file_cluster: {file_cluster_number}, sects per clust: {}", self.partition.start_sector, self.bpb.sectors_per_cluster);

                crate::println!("Found {} at sector {file_sector}", search);

                let data = self.drive.read(file_sector as u64, 1).unwrap();

                let str_data = &data[0..file_entry.file_size as usize];

                crate::println!("File data: {}", core::str::from_utf8(str_data).unwrap());

                break;
            } else {
                // Long file name
                if long_filename_string != Some(search.to_string()) {
                    continue;
                }

                crate::println!("Found file: {:?}", file_entry);

                let file_cluster = u32::from_le(file_entry.low_first_cluster_number as u32);
                let file_sector = self.partition.start_sector as usize
                    + self.cluster_to_sector(file_cluster as usize);

                // crate::println!("start: {} data: {first_data_sector} file_cluster: {file_cluster_number}, sects per clust: {}", self.partition.start_sector, self.bpb.sectors_per_cluster);

                crate::println!(
                    "Found {} at sector {file_sector} size: {}",
                    search,
                    (file_entry.file_size / 512) as usize
                );

                let data = self.drive.read(file_sector as u64, 1).unwrap();

                crate::println!("File data: {:X?}", data);

                break;
            }

            // crate::println!("{:X?}", file_entry);
        }

        // crate::println!("FAT: {:?}", fat);
        // crate::println!("{:?}", data_sector);
    }

    fn cluster_to_sector(&self, cluster: usize) -> usize {
        let fat_size = self.ebpb.sectors_per_fat;
        let root_dir_sectors = ((self.bpb.root_directory_count * 32)
            + (self.bpb.bytes_per_sector - 1))
            / self.bpb.bytes_per_sector;

        let first_data_sector = self.bpb.reserved_sectors as u32
            + (self.bpb.fat_count as u32 * fat_size)
            + root_dir_sectors as u32;

        return ((cluster - 2) as isize * self.bpb.sectors_per_cluster as isize) as usize
            + first_data_sector as usize;
    }
}
