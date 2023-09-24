use core::mem::size_of;

use crate::{
    arch::io::{inb, insw, inw, outb, outw},
    libs::mutex::Mutex,
};

const ATA_SECTORS: usize = 512;

#[repr(u8)]
enum ATADriveStatus {
    Error = 0x01,
    Index = 0x02,
    Corrupt = 0x04,
    DataReqReady = 0x08,
    DriveSeekDone = 0x10,
    WriteFault = 0x20,
    Ready = 0x40,
    Busy = 0x80,
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
    ide_initialize(0x1f0, 0x3F6, 0x170, 0x376, 0x000);
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

    pub fn is_busy(&self) -> bool {
        return self.status() & ATADriveStatus::Busy as u16 != 0;
    }

    pub fn is_ready(&self) -> bool {
        return self.status() & ATADriveStatus::DataReqReady as u16 != 0;
    }

    pub fn is_error(&self) -> bool {
        return self.status() & ATADriveStatus::Error as u16 != 0;
    }

    pub fn identify(&self, drive: u8) -> bool {
        self.select(drive);

        for i in 0..3 {
            outb(
                (self.io_port + ATADriveRegister::SectorCount0 as u16) + i,
                0,
            );
        }

        self.send_command(ATADriveCommand::Identify);

        if self.status() == 0 {
            // drive did not respond to identify command
            // therefore, the drive is not present
            return false;
        }

        while self.is_busy() {
            if self.is_ready() {
                break;
            }

            if self.is_error() {
                return false;
            }
        }

        for i in 0..256 {
            DRIVE_ID.lock().write()[drive as usize - ATADriveType::Parent as usize][i] =
                inw(self.io_port + ATADriveRegister::Data as u16)
        }

        return true;
    }

    pub fn is_present(&self, drive: u8) -> bool {
        self.select(drive);

        if self.status() == 0xFF {
            // The bus is not present
            return false;
        }

        return self.identify(drive);
    }

    pub fn read(
        &self,
        drive: ATADriveType,
        buffer: &mut [u8],
        sector: u64,
        sector_count: u16,
    ) -> bool {
        let selector = (drive as u8 + 0x20) | ((sector >> 24) & 0x0F) as u8;

        self.select(selector);

        while self.is_busy() {}

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

        let words = buffer.as_mut_ptr() as *mut u16;

        for _ in 0..sector_count {
            while self.is_busy() {}
            while !self.is_ready() {}

            if self.is_error() {
                return false;
            }

            insw(
                self.io_port + ATADriveRegister::Data as u16,
                words,
                ATA_SECTORS / size_of::<u16>(),
            );
        }

        return true;
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
            (sectors * ATA_SECTORS as u64) / 1024 / 1024
        )
    }

    let a = DRIVE_ID.lock().read()[0];

    crate::println!("{:?}", a);

    let mut buffer = [0u8; 512];

    drive.read(ATADriveType::Parent, buffer.as_mut(), 0, 1);

    crate::println!("{:X?}", buffer);
}
