use core::{alloc::Layout, mem::size_of};

use alloc::{alloc::alloc, alloc::dealloc, sync::Arc, vec::Vec};

use crate::{
    arch::io::{inb, insw, inw, outb, outw},
    libs::mutex::Mutex,
    log_info,
};

const ATA_SECTOR_SIZE: usize = 512;

#[repr(u8)]
#[derive(Debug, PartialEq)]
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
    ATA = 0x00,
    ATAPI = 0x01,
}

#[repr(u8)]
enum ATADriveType {
    Parent = 0xA0,
    Child = 0xB0,
}

#[repr(u8)]
enum ATADriveRegister {
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
    ControlAndAltStatus = 0x0C,
    // AltStatus = 0x0C,
    DeviceAddress = 0x0D,
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

struct IdeDevice {
    reserved: u8,
    channel: u8,
    drive: ATADriveType,
    drive_type: IDEDriveType,
    signature: u16,
    capabilities: u16,
    command_sets: u32,
    size: u32,
    model: [u8; 40],
    io_port: u16,
    control_port: u16,
}

struct Drive {
    io_port: u16,
    control_port: u16,
}

impl Drive {
    const fn new(io_port: u16, control_port: u16) -> Self {
        return Self {
            io_port,
            control_port,
        };
    }

    pub fn select(&self, drive: u8) {
        outb(
            self.io_port + ATADriveRegister::DeviceSelect as u16,
            drive as u8,
        );
    }

    pub fn send_command(&self, command: ATADriveCommand) {
        outb(
            self.io_port + ATADriveRegister::CommandAndStatus as u16,
            command as u8,
        )
    }

    pub fn status(&self) -> u16 {
        return inb(self.io_port + ATADriveRegister::CommandAndStatus as u16) as u16;
    }

    pub fn sectors(&self, drive: u8) -> u64 {
        let sectors = unsafe {
            DRIVE_ID.lock().read()[drive as usize - ATADriveType::Parent as usize]
                .as_ptr()
                .add(100)
        };

        return unsafe { *(sectors as *const u64) };
    }

    fn wait_for_drive_ready(&self) -> Result<(), ()> {
        loop {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            crate::arch::pause();

            let status = self.status();

            if status & ATADriveStatus::Error as u16 != 0
                || status & ATADriveStatus::WriteFault as u16 != 0
            {
                return Err(());
            }

            if status & ATADriveStatus::Busy as u16 != 0 {
                continue;
            }

            if status & ATADriveStatus::DataReqReady as u16 != 0 {
                return Ok(());
            }
        }
    }

    pub fn await_busy(&self) {
        while self.status() & ATADriveStatus::Busy as u16 != 0 {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            crate::arch::pause();
        }
    }

    pub fn identify(&self, drive: u8) -> Result<(), ()> {
        self.select(drive);

        for i in 0..3 {
            outb(
                (self.io_port + ATADriveRegister::SectorCount0 as u16) + i,
                0,
            );
        }

        self.send_command(ATADriveCommand::Identify);

        if self.status() as u8 == 0x00 {
            // drive did not respond to identify command
            // therefore, the drive is not present
            return Err(());
        }

        let _ = self
            .wait_for_drive_ready()
            .map_err(|_| crate::log_error!("Error identifying drive"))?;

        for i in 0..256 {
            DRIVE_ID.lock().write()[drive as usize - ATADriveType::Parent as usize][i] =
                inw(self.io_port + ATADriveRegister::Data as u16)
        }

        return Ok(());
    }

    pub fn is_present(&self, drive: u8) -> bool {
        self.select(drive);

        if self.status() == 0xFF {
            return false;
        }

        return self.identify(drive).is_ok();
    }

    pub fn read(&self, drive: ATADriveType, sector: u64, sector_count: u16) -> Result<Vec<u8>, ()> {
        let selector = (drive as u8 + 0x20) | ((sector >> 24) & 0x0F) as u8;

        self.select(selector);

        self.await_busy();

        let using_lba48 = sector >= (1 << 28) - 1;

        if using_lba48 {
            outw(
                self.io_port + ATADriveRegister::SectorCount0 as u16,
                sector_count,
            );
            outb(self.io_port + ATADriveRegister::LBA0 as u16, sector as u8);
            outb(
                self.io_port + ATADriveRegister::LBA1 as u16,
                (sector >> 8) as u8,
            );
            outb(
                self.io_port + ATADriveRegister::LBA2 as u16,
                (sector >> 16) as u8,
            );
            outb(
                self.io_port + ATADriveRegister::LBA3 as u16,
                (sector >> 24) as u8,
            );
            outb(self.io_port + ATADriveRegister::LBA4 as u16, 0);
            outb(self.io_port + ATADriveRegister::LBA5 as u16, 0);

            self.send_command(ATADriveCommand::ReadPIOExt);
        } else {
            crate::println!("LBA28");

            outw(
                self.io_port + ATADriveRegister::SectorCount0 as u16,
                sector_count,
            );
            outb(self.io_port + ATADriveRegister::LBA0 as u16, sector as u8);
            outb(
                self.io_port + ATADriveRegister::LBA1 as u16,
                (sector >> 8) as u8,
            );
            outb(
                self.io_port + ATADriveRegister::LBA2 as u16,
                (sector >> 16) as u8,
            );
            outb(self.io_port + ATADriveRegister::LBA3 as u16, 0);
            outb(self.io_port + ATADriveRegister::LBA4 as u16, 0);
            outb(self.io_port + ATADriveRegister::LBA5 as u16, 0);

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
                    self.io_port + ATADriveRegister::Data as u16,
                    (buffer.as_mut_ptr() as *mut u16)
                        .add((i as usize * ATA_SECTOR_SIZE) / size_of::<u16>()),
                    ATA_SECTOR_SIZE / size_of::<u16>() as usize,
                );
            }
        }

        unsafe {
            buffer.set_len(array_size);
        }

        return Ok(buffer);
    }
}

static IDE_DEVICES: Mutex<[bool; 2]> = Mutex::new([false; 2]);

// TODO: This code is pretty much just the C from @Moldytzu's mOS
// This code could probably be made better and more device agnostic
// But that's TODO obviously
fn ide_initialize(_bar0: u32, _bar1: u32, _bar2: u32, _bar3: u32, _bar4: u32) {
    let io_port_base = 0x1F0;
    let control_port_base = 0x3F6;

    let drive = Drive::new(io_port_base, control_port_base);

    (*IDE_DEVICES.lock().write()) = [
        drive.is_present(ATADriveType::Parent as u8),
        drive.is_present(ATADriveType::Child as u8),
    ];

    let ide_devices = IDE_DEVICES.lock().read();

    let drive_count = ide_devices[0] as u8 + ide_devices[1] as u8;

    crate::log_info!(
        "ATA: Detected {} drive{}",
        drive_count,
        match drive_count {
            1 => "",
            _ => "s",
        }
    );

    if ide_devices[0] {
        let sectors = drive.sectors(ATADriveType::Parent as u8);

        crate::log_info!(
            "ATA: Drive 0 has {} sectors ({} MB)",
            sectors,
            (sectors * ATA_SECTOR_SIZE as u64) / 1024 / 1024
        );

        let buffer = drive.read(ATADriveType::Parent, 0, 2);

        crate::println!("{:X?}", buffer);
    }
}
