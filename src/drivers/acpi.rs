use crate::{
    libs::{lazy::Lazy, mutex::Mutex},
    log_error, log_info, log_ok,
};
use alloc::{sync::Arc, vec::Vec};
use limine::RsdpRequest;

static RSDP_REQUEST: RsdpRequest = RsdpRequest::new(0);

#[repr(C, packed)]
struct Rsdp {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

#[repr(C, packed)]
struct Xsdp {
    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}

pub struct Acpi {
    tables: Arc<[*const u8]>,
    entries: usize,
    v2: bool,
}

impl Acpi {
    fn has_signature(&self, table_index: usize, signature: &str) -> bool {
        unsafe {
            let sdt_header: &SdtHeader = &*(self.tables[table_index] as *const SdtHeader);
            let st = core::str::from_utf8_unchecked(&sdt_header.signature);
            st == signature
        }
    }

    pub fn list_tables(&self) {
        unsafe {
            for i in 0..self.entries {
                let sdt_header: &SdtHeader = &*(self.tables[i] as *const SdtHeader);
                let st = sdt_header.signature;
                log_info!("entry {:02}: {:?}", i + 1, sdt_header);
            }
        }
    }

    pub fn get_table(&self, signature: &str) -> Option<*const u8> {
        for i in 0..self.entries {
            if self.has_signature(i, signature) {
                return Some(self.tables[i]);
            }
        }

        None
    }
}

const RSDP_V1_LENGTH: usize = 20;
const RSDP_V2_EXT_LENGTH: usize = core::mem::size_of::<Rsdp>() - RSDP_V1_LENGTH;
const RSDP_SIG: [u8; 8] = *b"RSD PTR ";

impl Rsdp {
    pub fn is_valid(&self) -> bool {
        if self.signature != RSDP_SIG {
            return false;
        }

        let bytes = unsafe { core::slice::from_raw_parts(self as *const Rsdp as *const u8, 20) };
        let sum = bytes.iter().fold(0u8, |sum, &byte| sum.wrapping_add(byte));

        if sum & 0xFF != 0 {
            return false;
        }

        return true;
    }
}

impl Xsdp {
    pub fn is_valid(&self) -> bool {
        let bytes = unsafe { core::slice::from_raw_parts(self as *const Xsdp as *const u8, 20) };
        let sum = bytes.iter().fold(0u8, |sum, &byte| sum.wrapping_add(byte));

        if sum & 0xFF != 0 {
            return false;
        }

        return true;
    }
}

static ACPI: Lazy<Mutex<Acpi>> = Lazy::new(|| {
    let acpi = resolve_acpi();

    if acpi.is_err() {
        panic!("Failed to resolve ACPI!");
    }

    Mutex::new(acpi.unwrap())
});

pub fn init_acpi() {
    let acpi_lock = ACPI.lock();
    let acpi = acpi_lock.read();

    log_ok!("Successfully initialized ACPI with {} tables", acpi.entries);

    acpi.list_tables()
}

#[derive(Debug)]
enum AcpiRootTable {
    Rsdp(u32),
    Xsdp(u64),
}

impl AcpiRootTable {
    pub fn get_from_bootloader() -> Result<Self, ()> {
        let rsdp_response = RSDP_REQUEST.get_response().get();

        if rsdp_response.is_none() {
            log_error!("Failed to initialize ACPI: RSDP not found!");
            return Err(());
        }

        let rsdp_address = &rsdp_response.unwrap().address;

        let rsdp_table: &Rsdp = unsafe { &*(rsdp_address.as_ptr().unwrap() as *const Rsdp) };

        if !rsdp_table.is_valid() {
            log_error!("Failed to initialize ACPI: RSDP was not valid!");
            return Err(());
        }

        if rsdp_table.revision == 2 {
            let xsdp_table: &Xsdp = unsafe { &*(rsdp_address.as_ptr().unwrap() as *const Xsdp) };

            if !xsdp_table.is_valid() {
                log_error!("Failed to initalize ACPI: XSDP was not valid!");
                return Err(());
            }

            return Ok(AcpiRootTable::Xsdp(xsdp_table.xsdt_address));
        }

        return Ok(AcpiRootTable::Rsdp(rsdp_table.rsdt_address));
    }
}

fn resolve_acpi() -> Result<Acpi, ()> {
    let root_table = AcpiRootTable::get_from_bootloader()?;

    crate::println!("{:?}", root_table);

    let (header_addr, ptr_size, ext_table) = match root_table {
        AcpiRootTable::Rsdp(addr) => (addr as u64, core::mem::size_of::<u32>(), false),
        AcpiRootTable::Xsdp(addr) => (addr, core::mem::size_of::<u64>(), true),
    };

    let root_header: &SdtHeader = unsafe { &*(header_addr as *const SdtHeader) };

    unsafe {
        if !ext_table {
            if core::str::from_utf8_unchecked(&root_header.signature) != "RSDT" {
                log_error!("Invalid root table header, expected RSDT.");
                return Err(());
            }
        } else {
            if core::str::from_utf8_unchecked(&root_header.signature) != "XSDT" {
                log_error!("Invalid root table header, expected XSDT.");
                return Err(());
            }
        }
    }

    let mut entries = (root_header.length as usize - core::mem::size_of::<SdtHeader>()) / ptr_size;
    if entries > 48 {
        log_error!("Expected at most 48 ACPI tables, got {entries}!");
        entries = 48;
    }

    let mut acpi_tables: Vec<*const u8> = Vec::with_capacity(entries);

    for i in 0..entries {
        let address =
            (header_addr + (core::mem::size_of::<SdtHeader>() + i * ptr_size) as u64) as *const u8;

        acpi_tables.push(address);
    }

    crate::println!("{:?} {}", acpi_tables, entries);

    return Ok(Acpi {
        tables: Arc::from(acpi_tables),
        entries,
        v2: ext_table,
    });
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct SdtHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}

fn check_rsdt_checksum(table_header: *const SdtHeader) -> bool {
    let mut sum: u8 = 0;

    for i in 0..unsafe { (*table_header).length } {
        sum += unsafe { *((table_header as *const u8).add(i as usize)) };
    }

    return sum == 0;
}

#[repr(C, packed)]
struct RSDT {
    h: SdtHeader,
    pointer_to_other_sdt: *const u8,
}

fn find_fadt(root_sdt: *const RSDT) -> Option<SdtHeader> {
    // unsafe {
    //     let rsdt = root_sdt.as_ref()?;
    //     let entries = (rsdt.h.length - core::mem::size_of::<AcpiSdtHeader>() as u32) / 4;

    //     let pointer_to_other_sdt =

    //     for i in 0..entries {
    //         crate::println!("{i}");

    //         let h_ptr = rsdt.pointer_to_other_sdt[i as usize] as *const AcpiSdtHeader;
    //         let h = h_ptr.as_ref()?;
    //         let slice = core::slice::from_raw_parts(h.signature.as_ptr(), 4);

    //         let signature = core::str::from_utf8(slice).ok()?;

    //         if signature == "FACP" {
    //             return Some(*h_ptr);
    //         }
    //     }

    //     // No FACP found
    //     return None;
    // }
    None
}
