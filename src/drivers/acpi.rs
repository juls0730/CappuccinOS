use alloc::vec::Vec;

use crate::{
    arch::io::{inw, outb},
    libs::oncecell::OnceCell,
};

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct SDTHeader {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oemid: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
}

#[repr(C, packed)]
#[derive(Debug)]
pub struct SDT<'a, T> {
    pub header: &'a SDTHeader,
    pub inner: &'a T,
    pub extra: Option<&'a [u8]>,
}

impl<'a, T> SDT<'a, T> {
    unsafe fn new(ptr: *const u8) -> Self {
        let length = core::ptr::read_unaligned(ptr.add(4).cast::<u32>());
        let data = core::slice::from_raw_parts(ptr, length as usize);

        crate::log_serial!("SDT at: {ptr:p}");

        assert!(data.len() == length as usize);

        let header: &SDTHeader = core::mem::transmute(data[0..].as_ptr());
        let inner: &T = core::mem::transmute(data[core::mem::size_of::<SDTHeader>()..].as_ptr());
        let mut extra = None;

        if length as usize > core::mem::size_of::<SDTHeader>() + core::mem::size_of::<T>() {
            extra = Some(&data[core::mem::size_of::<SDTHeader>() + core::mem::size_of::<T>()..]);
        }

        return Self {
            header,
            inner,
            extra,
        };
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct RSDP {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_addr: u32,
}

#[repr(C, packed)]
#[derive(Debug)]
struct XSDP {
    rsdp: RSDP,

    length: u32,
    xsdt_addr: u64,
    ext_checksum: u8,
    _reserved: [u8; 3],
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct RSDT {
    pointers: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct XSDT {
    pointers: u64,
}

#[derive(Debug)]
enum RootSDT<'a> {
    RSDT(SDT<'a, RSDT>),
    XSDT(SDT<'a, XSDT>),
}

impl<'a> RootSDT<'a> {
    fn header(&self) -> SDTHeader {
        return match self {
            RootSDT::RSDT(RSDT) => *RSDT.header,
            RootSDT::XSDT(XSDT) => *XSDT.header,
        };
    }

    fn len(&self) -> usize {
        let ptr_size = match self {
            &RootSDT::RSDT(_) => 4,
            &RootSDT::XSDT(_) => 8,
        };

        return (self.header().length as usize - core::mem::size_of::<SDTHeader>()) / ptr_size;
    }

    unsafe fn get(&self, idx: usize) -> *const u8 {
        let mut offset = 0;

        let root_ptr = match self {
            RootSDT::RSDT(RSDT) => {
                let ptrs = RSDT.inner.pointers as *const u8;
                assert!(!ptrs.is_null());
                ptrs.add(offset)
            }
            RootSDT::XSDT(XSDT) => {
                let ptrs = XSDT.inner.pointers as *const u8;
                assert!(!ptrs.is_null());
                ptrs.add(offset)
            }
        };

        for _ in 0..idx {
            let header: &SDTHeader = core::mem::transmute(root_ptr.add(offset).cast::<SDTHeader>());
            offset += header.length as usize;
        }

        return root_ptr.add(offset);
    }
}

#[derive(Debug)]
struct ACPI<'a> {
    root_sdt: RootSDT<'a>,
    tables: Vec<[u8; 4]>,
}

static ACPI: OnceCell<ACPI> = OnceCell::new();

static RSDP_REQ: limine::RsdpRequest = limine::RsdpRequest::new(0);

fn resolve_acpi() {
    let rsdp_ptr = RSDP_REQ.get_response().get();
    if rsdp_ptr.is_none() {
        panic!("RSDP not found!");
    }

    let rsdp = unsafe { &*rsdp_ptr.unwrap().address.as_ptr().unwrap().cast::<RSDP>() };

    // TODO: validate RSDT
    let root_sdt = {
        if rsdp.revision == 0 {
            RootSDT::RSDT(unsafe { SDT::new(rsdp.rsdt_addr as *mut u8) })
        } else {
            let xsdt = unsafe { &*rsdp_ptr.unwrap().address.as_ptr().unwrap().cast::<XSDP>() };
            RootSDT::XSDT(unsafe { SDT::new(xsdt.xsdt_addr as *mut u8) })
        }
    };

    let tables: Vec<[u8; 4]> = (0..root_sdt.len())
        .map(|i| {
            let sdt_ptr = unsafe { root_sdt.get(i) };
            let signature = unsafe { core::slice::from_raw_parts(sdt_ptr, 4) };
            let ret = signature.try_into().unwrap();
            ret
        })
        .collect();

    let acpi_table = ACPI { root_sdt, tables };

    ACPI.set(acpi_table);
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct GenericAddressStructure {
    address_space: u8,
    bit_width: u8,
    bit_offset: u8,
    access_size: u8,
    address: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct FADT {
    firmware_ctrl: u32,
    dsdt: u32,

    _reserved: u8,

    preferred_power_management_profile: u8,
    sci_interrupt: u16,
    smi_cmd_port: u32,
    acpi_enable: u8,
    acpi_disable: u8,
    s4biox_req: u8,
    pstate_control: u8,
    pm1a_event_block: u32,
    pm1b_event_block: u32,
    pm1a_control_block: u32,
    pm1b_control_block: u32,
    pm2_control_block: u32,
    pm_timer_block: u32,
    gpe0_block: u32,
    gpe1_block: u32,
    pm1_event_length: u8,
    pm1_control_length: u8,
    pm2_control_length: u8,
    pm_timer_length: u8,
    gpe0_length: u8,
    gpe1_length: u8,
    gpe1_base: u8,
    c_state_control: u8,
    worst_c2_latency: u16,
    worst_c3_length: u16,
    flush_size: u16,
    flush_stride: u16,
    duty_offset: u8,
    duty_width: u8,
    day_alarm: u8,
    month_alarm: u8,
    century: u8,

    boot_architecture_flags: u16,

    _reserved2: u8,
    flags: u32,

    reset_reg: GenericAddressStructure,

    reset_value: u8,
    _reserved3: [u8; 3],

    x_firmware_control: u64,
    x_dstd: u64,

    x_pm1a_event_block: GenericAddressStructure,
    x_pm1b_event_block: GenericAddressStructure,
    x_pm1a_control_block: GenericAddressStructure,
    x_pm1b_control_block: GenericAddressStructure,
    x_pm2_control_block: GenericAddressStructure,
    x_pm_timer_block: GenericAddressStructure,
    x_gpe0_block: GenericAddressStructure,
    x_gpe1_block: GenericAddressStructure,
}

pub fn init_acpi() {
    resolve_acpi();

    crate::log_ok!("Found {} ACPI Tables!", ACPI.tables.len());

    crate::log_serial!("Available serial tables:");
    for i in 0..ACPI.tables.len() {
        crate::log_serial!("    {}", core::str::from_utf8(&ACPI.tables[i]).unwrap())
    }

    let fadt = find_table::<FADT>("FACP").expect("Failed to find FADT");

    outb(fadt.inner.smi_cmd_port as u16, fadt.inner.acpi_enable);

    while inw(fadt.inner.pm1a_control_block as u16) & 1 == 0 {}

    crate::arch::interrupts::PICS.lock().write().disable();

    *crate::arch::apic::APIC.lock().write() =
        Some(crate::arch::apic::APIC::new().expect("Failed to enable APIC!"));

    crate::log_ok!("APIC enabled!");
}

pub fn find_table<T>(table_name: &str) -> Option<SDT<T>> {
    assert_eq!(table_name.len(), 4);

    for (i, table) in ACPI.tables.iter().enumerate() {
        if table == table_name.as_bytes() {
            let ptr = unsafe { ACPI.root_sdt.get(i) };

            let table = unsafe { SDT::new(ptr) };
            return Some(table);
        }
    }

    return None;
}
