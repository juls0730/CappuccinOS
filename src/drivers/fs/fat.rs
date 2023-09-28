use alloc::{
    format,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};

use crate::drivers::storage::drive::BlockDevice;

enum FatType {
    Fat12,
    Fat16,
    Fat32,
}

#[derive(Debug)]
pub struct BIOSParameterBlock {
    _jmp_instruction: [u8; 3],
    pub oem_identifier: [u8; 8],
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub fat_count: u8,
    pub root_directory_count: u16,
    pub total_sectors: u16, // equal to zero when sector count is more than 65535
    pub media_descriptor_type: u8,
    pub sectors_per_fat: u16, // Fat12/Fat16 only
    pub sectors_per_track: u16,
    pub head_count: u16,
    pub hidden_sectors: u32,
    pub large_sector_count: u32,
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

#[derive(Debug)]
pub struct ExtendedBIOSParameterBlock {
    pub sectors_per_fat: u32,
    pub flags: [u8; 2],
    pub fat_version: u16,
    pub root_dir_cluster: u32,
    pub fsinfo_sector: u16,
    pub backup_bootsector: u16,
    _reserved: [u8; 12],
    pub drive_number: u8,
    _reserved2: u8,
    pub signature: u8, // either 0x28 of 0x29
    pub volume_id: u32,
    pub volume_label: [u8; 11],
    _system_identifier_string: [u8; 8], // Always "FAT32   " but never trust the contents of this string (for some reason)
    _boot_code: [u8; 420],
    _bootable_signature: u16, // 0xAA55
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
