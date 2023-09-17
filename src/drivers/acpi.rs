use crate::{log_error, log_info, log_ok};
use limine::RsdpRequest;

static RSDP_REQUEST: RsdpRequest = RsdpRequest::new(0);

#[repr(C, packed)]
struct RSDP {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,

    // Only on Revision > 0
    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}

const RSDP_V1_LENGTH: usize = 20;
const RSDP_V2_EXT_LENGTH: usize = core::mem::size_of::<RSDP>() - RSDP_V1_LENGTH;
const RSDP_SIG: [u8; 8] = *b"RSD PTR ";

impl RSDP {
    pub fn is_valid(&self) -> bool {
        if self.signature != RSDP_SIG {
            return false;
        }

        if core::str::from_utf8(&self.oem_id).is_err() {
            return false;
        }

        let length = if self.revision > 0 {
            self.length as usize
        } else {
            RSDP_V1_LENGTH
        };

        let bytes =
            unsafe { core::slice::from_raw_parts(self as *const RSDP as *const u8, length) };
        let sum = bytes.iter().fold(0u8, |sum, &byte| sum.wrapping_add(byte));

        if sum != 0 {
            return false;
        }

        return true;
    }
}

pub fn init_acpi() {
    let rsdp_response = RSDP_REQUEST.get_response().get();

    if rsdp_response.is_none() {
        log_error!("Failed to initialize ACPI: RSDP not found!");
        return;
    }

    let rsdp_address = &rsdp_response.unwrap().address;

    let rsdp_table: &RSDP = unsafe { &*(rsdp_address.as_ptr().unwrap() as *const RSDP) };

    if !rsdp_table.is_valid() {
        log_error!("Failed to initialize ACPI: RSDP was not valid!");
        return;
    }

    log_info!("{}", rsdp_table.revision);
    let mut facp: Option<&ACPISDTHeader> = None;

    let rsdt_address = rsdp_table.rsdt_address;
    facp = find_facp(rsdt_address as *const u32, rsdp_table.revision);

    if facp.is_some() {
        log_ok!("Successfully found FADT");
    }
    log_ok!("Successfully initialized ACPI");
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct ACPISDTHeader {
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

fn check_rsdt_checksum(table_header: *const ACPISDTHeader) -> bool {
    let mut sum: u8 = 0;

    for i in 0..unsafe { (*table_header).length } {
        sum += unsafe { *((table_header as *const u8).add(i as usize)) };
    }

    return sum == 0;
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct RSDT {
    h: ACPISDTHeader,
    pointer_to_other_sdt: *const u32,
}

fn find_facp(root_sdt: *const u32, revision: u8) -> Option<&'static ACPISDTHeader> {
    let rsdt: &mut RSDT = unsafe { &mut *(root_sdt as *mut RSDT) };
    rsdt.pointer_to_other_sdt =
        [(rsdt.h.length - core::mem::size_of::<ACPISDTHeader>() as u32) / 4].as_ptr();

    let entry_bytes = if revision > 0 { 8 } else { 4 };

    let entries = (rsdt.h.length - core::mem::size_of::<ACPISDTHeader>() as u32) / entry_bytes;

    for i in 0..entries {
        crate::println!("{i}");
        let h = unsafe { rsdt.pointer_to_other_sdt.add(i as usize) as *const ACPISDTHeader };
        let signature_bytes = unsafe { (*h).signature };
        let signature_str = core::str::from_utf8(&signature_bytes).unwrap_or("");

        crate::println!("{} {:?} {:?}", signature_str, signature_bytes, b"FACP");

        // if signature_str == "FACP" {
        //     let facp_header = unsafe { &*(h as *const _) };
        //     return None;
        // }
    }

    return None;
}
