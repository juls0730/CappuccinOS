use core::mem::size_of;

use alloc::{boxed::Box, sync::Arc, vec::Vec};

use crate::{
    arch::io::{inb, insw, inw, outb},
    drivers::{
        fs::fat,
        storage::drive::{GPTBlock, GPTPartitionEntry},
    },
    libs::mutex::Mutex,
};

use super::drive::BlockDevice;

const ATA_SECTOR_SIZE: usize = 512;

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
enum ATADriveStatus {
    Error = 0x01,
    Index = 0x02,
    Corrupt = 0x04,
    DataReqReady = 0x08,
    DriveSeekDone = 0x10,
    WriteFault = 0x20,
    Ready = 0x40,
    Busy = 0x80,
    NotPresent = 0xFF,
}

// u8 == ATADriveStatus
impl PartialEq<ATADriveStatus> for u8 {
    fn eq(&self, other: &ATADriveStatus) -> bool {
        return self & (*other as u8) != 0;
    }
}

impl core::convert::From<u8> for ATADriveStatus {
    fn from(value: u8) -> Self {
        match value {
            0x01 => Self::Error,
            0x02 => Self::Index,
            0x04 => Self::Corrupt,
            0x08 => Self::DataReqReady,
            0x10 => Self::DriveSeekDone,
            0x20 => Self::WriteFault,
            0x40 => Self::Ready,
            0x80 => Self::Busy,
            _ => Self::NotPresent,
        }
    }
}

#[repr(u8)]
enum ATADriveError {
    AddressMarkNotFound = 0x01,
    Track0NotFound = 0x02,
    CommandAborted = 0x04,
    MediaChangeReq = 0x08,
    IDNotFound = 0x10,
    MediaChanged = 0x20,
    UncorrectableData = 0x40,
    BadBlock = 0x80,
}

#[repr(u8)]
enum ATADriveCommand {
    ReadPIO = 0x20,
    ReadPIOExt = 0x24,
    ReadDMA = 0xC8,
    ReadDMAExt = 0x25,
    WritePIO = 0x30,
    WritePIOExt = 0x34,
    WriteDMA = 0xCA,
    WriteDMAExt = 0x35,
    CacheFlush = 0xE7,
    CacheFlushExt = 0xEA,
    Packet = 0xA0,
    IdentifyPacket = 0xA1,
    Identify = 0xEC,
}

#[repr(u8)]
enum ATADriveIdentifyResponse {
    DeviceType = 0x00,
    Cylinders = 0x02,
    Heads = 0x06,
    Sectors = 0x0C,
    Serial = 0x14,
    Model = 0x36,
    Capabilities = 0x62,
    FieldValid = 0x6A,
    MaxLBA = 0x78,
    CommandSets = 0xA4,
    MaxLBAExt = 0xC8,
}

#[repr(u16)]
enum IDEDriveType {
    Pata,
    PataPi,
    Sata,
    SataPi,
}

impl IDEDriveType {
    /// Determines the ATA device type based on the values of the LBA mid and LBA high
    /// ports after an identify device command has been issued, but before the response has been read.
    fn from_lba(lba_mid: u8, lba_high: u8) -> Option<IDEDriveType> {
        match (lba_mid, lba_high) {
            (0x00, 0x00) => Some(IDEDriveType::Pata),
            (0x14, 0xEB) => Some(IDEDriveType::PataPi),
            (0x3C, 0xC3) => Some(IDEDriveType::Sata),
            (0x69, 0x96) => Some(IDEDriveType::SataPi),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
enum ATADriveType {
    Parent = 0 << 4,
    Child = 1 << 4,
}

#[repr(u8)]
enum ATADriveDataRegister {
    Data = 0x00,
    ErrorAndFeatures = 0x01,
    // Features = 0x01,
    SectorCount0 = 0x02,
    LBA0 = 0x03,
    LBA1 = 0x04,
    LBA2 = 0x05,
    DeviceSelect = 0x06,
    CommandAndStatus = 0x07,
    // Status = 0x07,
    SectorCount1 = 0x08,
    LBA3 = 0x09,
    LBA4 = 0x0A,
    LBA5 = 0x0B,
}

#[repr(u8)]
enum ATADriveControlRegister {
    ControlAndAltStatus = 0x02,
    DeviceAddress = 0x03,
}

#[repr(u8)]
enum ATADriveChannels {
    Primary = 0x00,
    Secondary = 0x01,
}

#[repr(u8)]
enum ATADriveDirection {
    Read = 0x00,
    Write = 0x01,
}

static DRIVE_ID: Mutex<[[u16; 256]; 2]> = Mutex::new([[0u16; 256]; 2]);

pub fn init() {
    // for pci_device in super::pci::PCI_DEVICES.lock().read() {
    //     if pci_device.class_code != 0x01 || pci_device.subclass_code != 0x01 {
    //         continue;
    //     }

    //     let (bar0, bar1, bar2, bar3, bar4, _) =
    //         super::pci::get_pci_bar_addresses(pci_device.bus, pci_device.device, pci_device.func);

    //     crate::println!(
    //         "bar0: {} bar1: {} bar2: {} bar3: {} bar4: {}",
    //         bar0,
    //         bar1,
    //         bar2,
    //         bar3,
    //         bar4
    //     );

    //     ide_initialize(bar0, bar1, bar2, bar3, bar4);
    // }
    // crate::println!("{:?}", ata_identify_drive(0xB0));
    ide_initialize(0x1F0, 0x3F6, 0x170, 0x376, 0x000);
}

#[derive(Debug)]
struct ATABus {
    io_bar: u16,
    control_bar: u16,
}

impl<'a> ATABus {
    fn new(io_bar: u16, control_bar: u16) -> Arc<Self> {
        let io_bar = io_bar & 0xFFFC;
        let control_bar = control_bar & 0xFFFC;

        return Arc::from(Self {
            io_bar,
            control_bar,
        });
    }

    pub fn select(&self, drive: u8) {
        outb(
            self.io_bar + ATADriveDataRegister::DeviceSelect as u16,
            drive as u8,
        );
    }

    pub fn send_command(&self, command: ATADriveCommand) {
        outb(
            self.io_bar + ATADriveDataRegister::CommandAndStatus as u16,
            command as u8,
        );
    }

    pub fn status(&self) -> u8 {
        // Waste 400ns
        for _ in 0..4 {
            inb(self.control_bar + ATADriveControlRegister::ControlAndAltStatus as u16);
        }

        return inb(self.io_bar + ATADriveDataRegister::CommandAndStatus as u16);
    }

    fn wait_for_drive_ready(&self) -> Result<(), ()> {
        loop {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            crate::arch::pause();

            let status = self.status();

            if status == ATADriveStatus::Error || status == ATADriveStatus::WriteFault {
                return Err(());
            }

            if status == ATADriveStatus::Busy {
                continue;
            }

            if status == ATADriveStatus::DataReqReady {
                return Ok(());
            }
        }
    }

    pub fn await_busy(&self) {
        while self.status() == ATADriveStatus::Busy {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            crate::arch::pause();
        }
    }

    pub fn identify(&self, drive: ATADriveType) -> Result<Arc<[u8; ATA_SECTOR_SIZE]>, ()> {
        self.select(0xA0 | drive as u8);

        outb(self.io_bar + ATADriveDataRegister::SectorCount0 as u16, 0);
        outb(self.io_bar + ATADriveDataRegister::LBA0 as u16, 0);
        outb(self.io_bar + ATADriveDataRegister::LBA1 as u16, 0);
        outb(self.io_bar + ATADriveDataRegister::LBA2 as u16, 0);

        self.send_command(ATADriveCommand::Identify);

        if self.status() == 0x00 {
            // drive did not respond to identify command
            // therefore, the drive is not present
            return Err(());
        }

        while self.status() == ATADriveStatus::Busy {
            let lba_mid = inb(self.io_bar + ATADriveDataRegister::LBA1 as u16);
            let lba_high = inb(self.io_bar + ATADriveDataRegister::LBA2 as u16);

            if lba_mid != 0 || lba_high != 0 {
                return Err(());
            }
        }

        let lba_mid = inb(self.io_bar + ATADriveDataRegister::LBA1 as u16);
        let lba_high = inb(self.io_bar + ATADriveDataRegister::LBA2 as u16);

        match IDEDriveType::from_lba(lba_mid, lba_high) {
            Some(IDEDriveType::Pata) => {
                // The only type we support, for now :tm:
            }
            _ => return Err(()),
        };

        let mut buffer = [0u8; ATA_SECTOR_SIZE];

        self.wait_for_drive_ready()
            .map_err(|_| crate::log_error!("Error before issuing Identify command."))?;

        for i in 0..(ATA_SECTOR_SIZE / size_of::<u16>()) {
            let word = inw(self.io_bar + ATADriveDataRegister::Data as u16);

            unsafe {
                *(buffer.as_mut_ptr() as *mut u16).add(i) = word;
            };
        }

        return Ok(Arc::from(buffer));
    }

    pub fn read(
        &self,
        drive: ATADriveType,
        sector: u64,
        sector_count: usize,
    ) -> Result<Arc<[u8]>, ()> {
        self.await_busy();

        let using_lba48 = sector >= (1 << 28) - 1;

        if using_lba48 {
            self.select(0x40 | (drive as u8));

            // High bytes
            outb(
                self.io_bar + ATADriveDataRegister::SectorCount0 as u16,
                (sector_count >> 8) as u8,
            );
            outb(
                self.io_bar + ATADriveDataRegister::LBA0 as u16,
                (sector >> 24) as u8,
            );
            outb(
                self.io_bar + ATADriveDataRegister::LBA1 as u16,
                (sector >> 32) as u8,
            );
            outb(
                self.io_bar + ATADriveDataRegister::LBA2 as u16,
                (sector >> 40) as u8,
            );

            // Low bytes
            outb(
                self.io_bar + ATADriveDataRegister::SectorCount0 as u16,
                sector_count as u8,
            );
            outb(
                self.io_bar + ATADriveDataRegister::LBA0 as u16,
                (sector >> 16) as u8,
            );
            outb(
                self.io_bar + ATADriveDataRegister::LBA1 as u16,
                (sector >> 8) as u8,
            );
            outb(
                self.io_bar + ATADriveDataRegister::LBA2 as u16,
                sector as u8,
            );

            self.send_command(ATADriveCommand::ReadPIOExt);
        } else {
            self.select(0xE0 | (drive as u8) | ((sector >> 24) as u8 & 0x0F));

            outb(
                self.io_bar + ATADriveDataRegister::SectorCount0 as u16,
                sector_count as u8,
            );
            outb(
                self.io_bar + ATADriveDataRegister::LBA0 as u16,
                sector as u8,
            );
            outb(
                self.io_bar + ATADriveDataRegister::LBA1 as u16,
                (sector >> 8) as u8,
            );
            outb(
                self.io_bar + ATADriveDataRegister::LBA2 as u16,
                (sector >> 16) as u8,
            );

            self.send_command(ATADriveCommand::ReadPIO);
        }

        // sector count * 512 = bytes in array
        let array_size = (sector_count as usize) * ATA_SECTOR_SIZE;

        // Allocate memory for the array that stores the sector data
        let mut buffer = Vec::new();
        buffer.reserve_exact(array_size);

        for i in 0..sector_count {
            self.wait_for_drive_ready()
                .map_err(|_| crate::log_error!("Error reading IDE Device"))?;

            // # Safety
            //
            // We know that buffer is the exact size of count, so it will never panic:tm:
            unsafe {
                insw(
                    self.io_bar + ATADriveDataRegister::Data as u16,
                    (buffer.as_mut_ptr() as *mut u16)
                        .add((i as usize * ATA_SECTOR_SIZE) / size_of::<u16>()),
                    ATA_SECTOR_SIZE / size_of::<u16>() as usize,
                );

                buffer.set_len(buffer.len() + ATA_SECTOR_SIZE);
            }
        }

        // Turn Vec into Arc in favor of Arc's constant time copy
        let arc_data: Arc<[u8]> = Arc::from(buffer);

        return Ok(arc_data);
    }

    fn software_reset(&self) {
        // Procedure is (1) set the SRST bit, (2) wait 5us, (3) clear the SRST bit.
        outb(
            self.io_bar + ATADriveControlRegister::ControlAndAltStatus as u16,
            0x04,
        );
        // We wait 5us by reading the status port 50 times (each read takes 100ns)
        for _ in 0..10 {
            self.status(); // reads status port 5 times.
        }

        outb(
            self.io_bar + ATADriveControlRegister::ControlAndAltStatus as u16,
            0x00,
        );
    }
}

#[derive(Debug)]
struct ATADrive {
    bus: Arc<ATABus>,
    identify_data: Arc<[u8; ATA_SECTOR_SIZE]>,
    drive_type: ATADriveType,
}

impl ATADrive {
    pub fn new(bus: Arc<ATABus>, drive: ATADriveType) -> Result<Self, ()> {
        let identify_data = bus.identify(drive)?;

        let capabilities_bytes = &identify_data[98..100];

        assert_eq!(capabilities_bytes.len(), 2);

        let capabilities = (capabilities_bytes[0] as u16) | ((capabilities_bytes[1] as u16) << 8);

        if capabilities & 0x200 == 0 {
            // Old AF CHS Drive, just ignore it
            // for now:tm:
            return Err(());
        }

        return Ok(Self {
            bus,
            identify_data,
            drive_type: drive,
        });
    }
}

impl BlockDevice for ATADrive {
    fn read(&self, sector: u64, sector_count: usize) -> Result<Arc<[u8]>, ()> {
        if (sector + sector_count as u64) > self.sector_count() as u64 {
            return Err(());
        }

        self.bus.software_reset();

        return self.bus.read(self.drive_type, sector, sector_count);
    }

    fn sector_count(&self) -> u64 {
        let sectors = self.identify_data[120..].as_ptr();

        return unsafe { *(sectors as *const u32) } as u64;
    }

    fn write(&self, sector: u64, data: &[u8]) -> Result<(), ()> {
        panic!("TODO: ATA Drive writes");
    }
}

static DRIVES: Mutex<Vec<ATADrive>> = Mutex::new(Vec::new());

// TODO: This code is pretty much just the C from @Moldytzu's mOS
// This code could probably be made better and more device agnostic
// But that's TODO obviously
fn ide_initialize(bar0: u32, bar1: u32, _bar2: u32, _bar3: u32, _bar4: u32) {
    let io_port_base = bar0 as u16;
    let control_port_base = bar1 as u16;

    let bus = ATABus::new(io_port_base, control_port_base);

    for i in 0..2 {
        let drive_type = if i == 0 {
            ATADriveType::Parent
        } else {
            ATADriveType::Child
        };

        let drive = ATADrive::new(bus.clone(), drive_type);

        if drive.is_ok() {
            DRIVES.lock().write().push(drive.unwrap());
        }
    }

    crate::log_info!(
        "ATA: Detected {} drive{}",
        DRIVES.lock().read().len(),
        match DRIVES.lock().read().len() {
            1 => "",
            _ => "s",
        }
    );

    for drive in DRIVES.lock().read().iter() {
        let sectors = drive.sector_count();

        crate::log_info!(
            "ATA: Drive 0 has {} sectors ({} MB)",
            sectors,
            (sectors as u64 * ATA_SECTOR_SIZE as u64) / 1024 / 1024
        );

        let mbr_sector = drive.read(0, 1).expect("Failed to read first sector");

        let signature: [u8; 2] = mbr_sector[510..].try_into().unwrap();

        if u16::from_le_bytes(signature[0..2].try_into().unwrap()) != 0xAA55 {
            panic!("First sector is not MBR");
        }

        let gpt_sector = drive.read(1, 1).expect("Failed to read sector 2");

        let mut array = [0u8; 512];
        array.copy_from_slice(&gpt_sector[..512]);

        let mut signature = [0u8; 8];
        signature.copy_from_slice(&gpt_sector[0..8]);

        if &signature != b"EFI PART" {
            panic!("MBR Disk is unsupported!")
        }

        let gpt = GPTBlock::new(&array);

        let mut partitions: Vec<GPTPartitionEntry> =
            Vec::with_capacity(gpt.partition_entry_count as usize);

        let partition_sector = drive
            .read(
                2,
                (gpt.partition_entry_count * gpt.partition_entry_size) as usize / ATA_SECTOR_SIZE,
            )
            .expect("Failed to read partition table");

        for i in 0..gpt.partition_entry_count {
            let entry_offset = (i * gpt.partition_entry_size) as usize;

            let partition_type_guid: [u8; 16] = partition_sector[entry_offset..entry_offset + 16]
                .try_into()
                .unwrap();

            let mut is_zero = true;

            for &j in partition_type_guid.iter() {
                if j != 0 {
                    is_zero = false;
                }
            }

            if is_zero {
                continue;
            }

            let start_sector = u64::from_le_bytes(
                partition_sector[entry_offset + 32..entry_offset + 40]
                    .try_into()
                    .unwrap(),
            );
            let end_sector = u64::from_le_bytes(
                partition_sector[entry_offset + 40..entry_offset + 48]
                    .try_into()
                    .unwrap(),
            );

            // Store the parsed information in the partition_entries array
            partitions.push(GPTPartitionEntry {
                partition_type_guid,
                start_sector,
                end_sector,
            });
        }

        for &partition in partitions.iter() {
            let fat_fs = fat::FATFS::new(drive, partition);

            if fat_fs.is_err() {
                continue;
            }

            let fat_fs = fat_fs.unwrap();

            let vfs = crate::drivers::fs::vfs::VFS::new(Box::new(fat_fs));

            crate::drivers::fs::vfs::VFS_INSTANCES
                .lock()
                .write()
                .push(vfs);
        }

        crate::println!("{:?}", partitions);
    }
}
