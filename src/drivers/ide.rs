// use core::arch::asm;

// use crate::{
//     arch::io::{inb, insl, outb},
//     libs::mutex::Mutex,
//     log_error, log_ok,
// };

// #[repr(u8)]
// enum ATADriveStatus {
//     Error = 0x01,
//     Index = 0x02,
//     Corrupt = 0x04,
//     DataReqReady = 0x08,
//     DriveSeekDone = 0x10,
//     WriteFault = 0x20,
//     Ready = 0x40,
//     Busy = 0x80,
// }

// #[repr(u8)]
// enum ATADriveError {
//     AddressMarkNotFound = 0x01,
//     Track0NotFound = 0x02,
//     CommandAborted = 0x04,
//     MediaChangeReq = 0x08,
//     IDNotFound = 0x10,
//     MediaChanged = 0x20,
//     UncorrectableData = 0x40,
//     BadBlock = 0x80,
// }

// #[repr(u8)]
// enum ATADriveCommand {
//     ReadPIO = 0x20,
//     ReadPIOExt = 0x24,
//     ReadDMA = 0xC8,
//     ReadDMAExt = 0x25,
//     WritePIO = 0x30,
//     WritePIOExt = 0x34,
//     WriteDMA = 0xCA,
//     WriteDMAExt = 0x35,
//     CacheFlush = 0xE7,
//     CacheFlushExt = 0xEA,
//     Packet = 0xA0,
//     IdentifyPacket = 0xA1,
//     Identify = 0xEC,
// }

// #[repr(u8)]
// enum ATADriveIdentifyResponse {
//     DeviceType = 0x00,
//     Cylinders = 0x02,
//     Heads = 0x06,
//     Sectors = 0x0C,
//     Serial = 0x14,
//     Model = 0x36,
//     Capabilities = 0x62,
//     FieldValid = 0x6A,
//     MaxLBA = 0x78,
//     CommandSets = 0xA4,
//     MaxLBAExt = 0xC8,
// }

// #[repr(u8)]
// enum IDEDriveType {
//     ATA = 0x00,
//     ATAPI = 0x01,
// }

// #[repr(u8)]
// enum ATADriveType {
//     Master = 0x00,
//     Slave = 0x01,
// }

// #[repr(u8)]
// enum ATADriveRegister {
//     Data = 0x00,
//     Error = 0x01,
//     // Features = 0x01,
//     SectorCount0 = 0x02,
//     LBA0 = 0x03,
//     LBA1 = 0x04,
//     LBA2 = 0x05,
//     HDDeviceSelect = 0x06, // maybe incorrect name, idk
//     Command = 0x07,
//     // Status = 0x07,
//     SectorCount1 = 0x08,
//     LBA3 = 0x09,
//     LBA4 = 0x0A,
//     LBA5 = 0x0B,
//     Control = 0x0C,
//     // AltStatus = 0x0C,
//     DeviceAddress = 0x0D,
// }

// #[repr(u8)]
// enum ATADriveChannels {
//     Primary = 0x00,
//     Secondary = 0x01,
// }

// #[repr(u8)]
// enum ATADriveDirection {
//     Read = 0x00,
//     Write = 0x01,
// }

// #[derive(Clone, Copy)]
// struct IDEChannelRegisters {
//     base: u16,
//     ctrl: u16,
//     bmide: u16,
//     no_int: u8,
// }

// impl IDEChannelRegisters {
//     const fn new() -> Self {
//         return Self {
//             base: 0,
//             ctrl: 0,
//             bmide: 0,
//             no_int: 0,
//         };
//     }
// }

// static CHANNELS: Mutex<[IDEChannelRegisters; 2]> = Mutex::new([IDEChannelRegisters::new(); 2]);
// static IDE_BUF: Mutex<[u8; 2048]> = Mutex::new([0u8; 2048]);
// static IDE_IRQ_INVOKED: Mutex<u8> = Mutex::new(0);
// static ATAPI_PACKET: [u8; 12] = [0xA8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

// #[derive(Clone, Copy, Debug)]
// struct IDEDevice {
//     reserved: u8,
//     channel: u8,
//     drive: u8,
//     drive_type: u16,
//     signature: u16,
//     capabilities: u16,
//     command_sets: u32,
//     size: u32,
//     model: [u8; 41],
// }

// impl IDEDevice {
//     const fn new() -> Self {
//         return Self {
//             reserved: 0,
//             channel: 0,
//             drive: 0,
//             drive_type: 0,
//             signature: 0,
//             capabilities: 0,
//             command_sets: 0,
//             size: 0,
//             model: [0u8; 41],
//         };
//     }
// }

// static IDE_DEVICES: Mutex<[IDEDevice; 4]> = Mutex::new([IDEDevice::new(); 4]);

// fn ide_read(channel: u8, reg: u8) -> u8 {
//     let mut result: u8 = 0;
//     let reg = reg as u16;
//     let device = CHANNELS.lock().read()[channel as usize];

//     if reg > 0x07 && reg < 0x0C {
//         ide_write(
//             channel,
//             ATADriveRegister::Control as u8,
//             0x80 | device.no_int,
//         );
//     }

//     if reg < 0x08 {
//         result = inb(device.base + reg - 0x00);
//     } else if reg < 0x0C {
//         result = inb(device.base + reg - 0x06);
//     } else if reg < 0x0E {
//         result = inb(device.ctrl + reg - 0x0A);
//     } else if reg < 0x16 {
//         result = inb(device.bmide + reg - 0x0E);
//     }

//     if reg > 0x07 && reg < 0x0C {
//         ide_write(channel, ATADriveRegister::Control as u8, device.no_int);
//     }

//     return result;
// }

// fn ide_write(channel: u8, reg: u8, data: u8) {
//     let reg = reg as u16;
//     let device = CHANNELS.lock().read()[channel as usize];

//     if reg > 0x07 && reg < 0x0C {
//         ide_write(
//             channel,
//             ATADriveRegister::Control as u8,
//             0x80 | device.no_int,
//         );
//     }

//     if reg < 0x08 {
//         outb(device.base + reg - 0x00, data);
//     } else if reg < 0x0C {
//         outb(device.base + reg - 0x06, data);
//     } else if reg < 0x0E {
//         outb(device.ctrl + reg - 0x0A, data);
//     } else if reg < 0x16 {
//         outb(device.bmide + reg - 0x0E, data);
//     }

//     if reg > 0x07 && reg < 0x0C {
//         ide_write(channel, ATADriveRegister::Control as u8, device.no_int);
//     }
// }

// fn ide_read_buffer(channel: u8, reg: u8, buffer: *mut u32, quads: u32) {
//     let reg = reg as u16;
//     let device = CHANNELS.lock().read()[channel as usize];

//     if reg > 0x07 && reg < 0x0C {
//         ide_write(
//             channel,
//             ATADriveRegister::Control as u8,
//             0x80 | device.no_int,
//         );
//     }

//     // unsafe {
//     //     asm!("push es", "mov ds, ax", "mov ax, es", options(nostack),);
//     // }

//     if reg < 0x08 {
//         insl(device.base + reg - 0x00, buffer, quads);
//     } else if reg < 0x0C {
//         insl(device.base + reg - 0x06, buffer, quads);
//     } else if reg < 0x0E {
//         insl(device.ctrl + reg - 0x0A, buffer, quads);
//     } else if reg < 0x16 {
//         insl(device.bmide + reg - 0x0E, buffer, quads);
//     }

//     // unsafe {
//     //     asm!("pop es");
//     // }

//     if reg > 0x07 && reg < 0x0C {
//         ide_write(channel, ATADriveRegister::Control as u8, device.no_int);
//     }
// }

// fn ide_polling(channel: u8, advanced_check: bool) -> u8 {
//     // (I) Delay 400 nanoseconds for BSY to be set:
//     for _ in 0..4 {
//         ide_read(channel, ATADriveRegister::Control as u8); // Reading the Alternate Status port wastes 100ns; loop four times.
//     }

//     // (II) Wait for BSY to be cleared:
//     while ide_read(channel, ATADriveRegister::Command as u8) & ATADriveStatus::Busy as u8 != 0 {
//         // Wait for BSY to be zero.
//     }

//     if advanced_check {
//         let state = ide_read(channel, ATADriveRegister::Command as u8); // Read Status Register.

//         // (III) Check For Errors:
//         if state & ATADriveStatus::Error as u8 != 0 {
//             return 2; // Error.
//         }

//         // (IV) Check If Device fault:
//         if state & ATADriveStatus::WriteFault as u8 != 0 {
//             return 1; // Device Fault.
//         }

//         // (V) Check DRQ:
//         // BSY = 0; DF = 0; ERR = 0, so we should check for DRQ now.
//         if state & ATADriveStatus::DataReqReady as u8 == 0 {
//             return 3; // DRQ should be set.
//         }
//     }

//     return 0; // No Error.
// }

// fn ide_initialize(bar0: u32, bar1: u32, bar2: u32, bar3: u32, bar4: u32) {
//     let mut k = 0;
//     let mut count = 0;

//     {
//         let mut channels_lock = CHANNELS.lock();
//         let channels = channels_lock.write();

//         channels[ATADriveChannels::Primary as usize].base =
//             ((bar0 & 0xFFFFFFFC) + 0x1F0 * (!bar0)) as u16;
//         channels[ATADriveChannels::Primary as usize].ctrl =
//             ((bar1 & 0xFFFFFFFC) + 0x3F6 * (!bar1)) as u16;
//         channels[ATADriveChannels::Secondary as usize].base =
//             ((bar2 & 0xFFFFFFFC) + 0x170 * (!bar2)) as u16;
//         channels[ATADriveChannels::Secondary as usize].ctrl =
//             ((bar3 & 0xFFFFFFFC) + 0x376 * (!bar3)) as u16;
//         channels[ATADriveChannels::Primary as usize].bmide = ((bar4 & 0xFFFFFFFC) + 0) as u16; // Bus Master IDE
//         channels[ATADriveChannels::Secondary as usize].bmide = ((bar4 & 0xFFFFFFFC) + 8) as u16;
//         // Bus Master IDE
//     }

//     // 2- Disable IRQs:
//     ide_write(
//         ATADriveChannels::Primary as u8,
//         ATADriveRegister::Control as u8,
//         2,
//     );
//     ide_write(
//         ATADriveChannels::Secondary as u8,
//         ATADriveRegister::Control as u8,
//         2,
//     );

//     let wait = || outb(0x80, 0);
//     for i in 0..2 {
//         for j in 0..2 {
//             let mut ide_devices_lock = IDE_DEVICES.lock();
//             let ide_devices = ide_devices_lock.write();

//             let mut err: u8 = 0;
//             let mut drive_type: u8 = IDEDriveType::ATA as u8;
//             let mut status: u8 = 0;

//             ide_devices[count].reserved = 0;

//             ide_write(i, ATADriveRegister::HDDeviceSelect as u8, 0xA0 | (j << 4));
//             wait();

//             if ide_read(i, ATADriveRegister::Command as u8) == 0 {
//                 continue;
//             }

//             loop {
//                 status = ide_read(i, ATADriveRegister::Command as u8);

//                 crate::println!("{status}");

//                 if status & ATADriveStatus::Error as u8 != 0 {
//                     err = 1;
//                     break;
//                 }

//                 if !(status & ATADriveStatus::Busy as u8) != 0
//                     && status & ATADriveStatus::DataReqReady as u8 != 0
//                 {
//                     break;
//                 }
//             }

//             if err != 0 {
//                 log_error!("ATA drive error {err}");

//                 let cl = ide_read(i, ATADriveRegister::LBA1 as u8);
//                 let ch = ide_read(i, ATADriveRegister::LBA2 as u8);

//                 if cl == 0x14 && ch == 0xEB {
//                     drive_type = IDEDriveType::ATAPI as u8;
//                 } else if cl == 0x69 && ch == 0x96 {
//                     drive_type = IDEDriveType::ATAPI as u8;
//                 } else {
//                     log_error!("Not an IDE Device");
//                     continue;
//                 }

//                 ide_write(
//                     i,
//                     ATADriveRegister::Command as u8,
//                     ATADriveCommand::IdentifyPacket as u8,
//                 );
//                 wait();
//             }

//             ide_read_buffer(
//                 i,
//                 ATADriveRegister::Command as u8,
//                 IDE_BUF.lock().write().as_mut_ptr() as *mut u32,
//                 128,
//             );

//             ide_devices[count].reserved = 1;
//             ide_devices[count].drive_type = drive_type as u16;
//             ide_devices[count].channel = i;
//             ide_devices[count].drive = j;
//             unsafe {
//                 ide_devices[count].signature = *(IDE_BUF
//                     .lock()
//                     .read()
//                     .as_ptr()
//                     .add(ATADriveIdentifyResponse::DeviceType as usize)
//                     as *const u16);
//                 ide_devices[count].capabilities = *(IDE_BUF
//                     .lock()
//                     .read()
//                     .as_ptr()
//                     .add(ATADriveIdentifyResponse::Capabilities as usize)
//                     as *const u16);
//                 ide_devices[count].command_sets = *(IDE_BUF
//                     .lock()
//                     .read()
//                     .as_ptr()
//                     .add(ATADriveIdentifyResponse::CommandSets as usize)
//                     as *const u32);
//             }

//             if (ide_devices[count].command_sets & (1 << 26)) != 0 {
//                 unsafe {
//                     ide_devices[count].size = *(IDE_BUF
//                         .lock()
//                         .read()
//                         .as_ptr()
//                         .add(ATADriveIdentifyResponse::MaxLBAExt as usize)
//                         as *const u32);
//                 };
//             } else {
//                 unsafe {
//                     ide_devices[count].size = *(IDE_BUF
//                         .lock()
//                         .read()
//                         .as_ptr()
//                         .add(ATADriveIdentifyResponse::MaxLBA as usize)
//                         as *const u32);
//                 };
//             }

//             while k < 40 {
//                 ide_devices[count].model[k] =
//                     IDE_BUF.lock().read()[ATADriveIdentifyResponse::Model as usize + k + 1];
//                 ide_devices[count].model[k + 1] =
//                     IDE_BUF.lock().read()[ATADriveIdentifyResponse::Model as usize + k];
//                 k += 2;
//             }
//             ide_devices[count].model[40] = 0;

//             count += 1;
//         }

//         for &ide_device in IDE_DEVICES.lock().read().iter() {
//             if ide_device.reserved == 1 {
//                 log_ok!("Found IDE device! {:?}", ide_device);
//             }
//         }
//     }
// }

use crate::arch::io::{inb, inw, outb};

static mut drive_id: [[u16; 256]; 2] = [[0u16; 256]; 2];

pub fn init() {
    // for pci_device in super::pci::PCI_DEVICES.lock().read() {
    //     if pci_device.class_code != 0x01 && pci_device.subclass_code != 0x01 {
    //         continue;
    //     }

    //     let (bar0, bar1, bar2, bar3, bar4, _) =
    //         super::pci::get_pci_bar_addresses(pci_device.bus, pci_device.device, pci_device.func);

    //     ide_initialize(bar0, bar1, bar2, bar3, bar4);
    // }
    crate::println!("{:?}", ata_identify_drive(0xB0));
}

fn ata_drive_status() -> u8 {
    return inb(0x1F7);
}

fn ata_select_drive(drive: u8) {
    outb(0x1F6, drive);
}

fn ata_send_command(command: u8) {
    outb(0x1F7, command);
}

fn ata_identify_drive(drive: u8) -> bool {
    ata_select_drive(drive);

    for i in 0..3 {
        outb(0x1F2 + i, 0);
    }

    ata_send_command(0xEC);

    if ata_drive_status() == 0 {
        return false;
    }

    while (ata_drive_status() & 0x80) != 0 {
        if (ata_drive_status() & 0x08) != 0 {
            break;
        }

        if (ata_drive_status() & 0x01) != 0 {
            return false;
        }
    }

    for i in 0..256 {
        unsafe {
            drive_id[(drive - 0xA0) as usize][i] = inw(0x1F0);
        }
    }

    return true;
}
