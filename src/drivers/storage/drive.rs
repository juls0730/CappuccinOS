use alloc::sync::Arc;

pub trait BlockDevice {
    fn sector_count(&self) -> u64;
    fn read(&self, sector: u64, sector_count: usize) -> Result<Arc<[u8]>, ()>;
    fn write(&self, sector: u64, data: &[u8]) -> Result<(), ()>;
}

#[derive(Debug, Default)]
pub struct GPTPartitionEntry {
    pub partition_type_guid: [u8; 16],
    pub start_sector: u64,
    pub end_sector: u64,
}

#[derive(Debug)]
pub struct GPTBlock {
    pub header: [u8; 8], // 0x45 0x46 0x49 0x20 0x50 0x41 0x52 0x54
    pub revision: u32,
    pub header_size: u32,
    pub header_checksum: u32, // CRC32
    _reserved: [u8; 4],
    pub header_lba: u64,
    pub header_lba_alt: u64,
    pub first_usable_block: u64,
    pub last_usable_block: u64,
    pub guid: [u8; 16],
    pub guid_lba: u64,
    pub partition_entry_count: u32,
    pub partition_entry_size: u32,
    pub partition_table_crc: u32,
    _reserved2: [u8; 512 - 0x5C],
}

impl GPTBlock {
    pub fn new(data: &[u8; 512]) -> Self {
        let header = data[0x00..0x08].try_into().unwrap();
        let revision = u32::from_le_bytes(data[0x08..0x0C].try_into().unwrap());
        let header_size = u32::from_le_bytes(data[0x0C..0x10].try_into().unwrap());
        let header_checksum = u32::from_le_bytes(data[0x10..0x14].try_into().unwrap());
        let _reserved = data[0x14..0x18].try_into().unwrap();
        let header_lba = u64::from_le_bytes(data[0x18..0x20].try_into().unwrap());
        let header_lba_alt = u64::from_le_bytes(data[0x20..0x28].try_into().unwrap());
        let first_usable_block = u64::from_le_bytes(data[0x28..0x30].try_into().unwrap());
        let last_usable_block = u64::from_le_bytes(data[0x30..0x38].try_into().unwrap());
        let guid = data[0x38..0x48].try_into().unwrap();
        let guid_lba = u64::from_le_bytes(data[0x48..0x50].try_into().unwrap());
        let partition_entry_count = u32::from_le_bytes(data[0x50..0x54].try_into().unwrap());
        let partition_entry_size = u32::from_le_bytes(data[0x54..0x58].try_into().unwrap());
        let partition_table_crc = u32::from_le_bytes(data[0x58..0x5C].try_into().unwrap());
        let _reserved2 = data[0x5C..512].try_into().unwrap();

        GPTBlock {
            header,
            revision,
            header_size,
            header_checksum,
            _reserved,
            header_lba,
            header_lba_alt,
            first_usable_block,
            last_usable_block,
            guid,
            guid_lba,
            partition_entry_count,
            partition_entry_size,
            partition_table_crc,
            _reserved2,
        }
    }
}
