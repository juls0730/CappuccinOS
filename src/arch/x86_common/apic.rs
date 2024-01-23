use core::sync::atomic::AtomicBool;

use alloc::{boxed::Box, sync::Arc, vec::Vec};

use crate::{drivers::acpi::SDTHeader, libs::mutex::Mutex};

use super::{cpu_get_msr, cpu_set_msr, pic::ChainedPics};

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct MADT {
    pub local_apic_address: u32,
    pub flags: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct MADTEntry {}

impl MADT {
    pub fn as_ptr(&self) -> *const u8 {
        core::ptr::addr_of!(self).cast::<u8>()
    }
}

const IA32_APIC_BASE_MSR: u32 = 0x1B;
const IA32_APIC_BASE_MSR_ENABLE: usize = 0x800;

pub fn has_apic() -> bool {
    return unsafe { core::arch::x86_64::__cpuid_count(1, 0).edx } & 1 << 9 != 0;
}

fn set_apic_base(apic: usize) {
    let edx: u32 = 0;
    let eax = (apic & 0xfffff0000) | IA32_APIC_BASE_MSR_ENABLE;

    unsafe { cpu_set_msr(IA32_APIC_BASE_MSR, &(eax as u32), &edx) };
}

fn get_apic_base() -> u32 {
    let mut eax: u32 = 0;
    let mut edx: u32 = 0;
    unsafe { cpu_get_msr(IA32_APIC_BASE_MSR, &mut eax, &mut edx) };

    return eax & 0xfffff000;
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct LAPIC {
    pub acpi_processor_id: u8,
    pub apic_id: u8,
    pub flags: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct IOAPIC {
    pub ioapic_id: u8,
    _reserved: u8,
    pub ptr: *mut u8,
    pub global_interrupt_base: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct IOAPICSourceOverride {
    bus_source: u8,
    irq_source: u8,
    global_system_interrupt: u32,
    flags: u16,
}

#[derive(Debug)]
pub struct APIC {
    pub io_apic: IOAPIC,
    local_apic: *mut u8,
    pub cpus: Arc<[LAPIC]>,
}

impl APIC {
    pub fn new() -> Result<Self, ()> {
        if !has_apic() {
            return Err(());
        }

        let apic_base = get_apic_base() as usize;

        set_apic_base(apic_base);

        let madt = crate::drivers::acpi::find_table::<MADT>("APIC");

        if madt.is_none() {
            return Err(());
        }

        let mut cpus: Vec<LAPIC> = Vec::new();

        let madt = madt.unwrap();

        crate::log_info!("MADT located at: {:p}", core::ptr::addr_of!(madt));

        let mut lapic_ptr = madt.inner.local_apic_address as *mut u8;
        let mut io_apic = None;
        let mut io_apic_source_override = None;

        let mut ptr = madt.extra.unwrap().as_ptr();
        let ptr_end = unsafe { ptr.add(madt.header.length as usize - 44) };

        while (ptr as usize) < (ptr_end as usize) {
            match unsafe { *ptr } {
                0 => {
                    if unsafe { *(ptr.add(4)) } & 1 != 0 {
                        cpus.push(unsafe { *ptr.add(2).cast::<LAPIC>() });
                    }
                }
                1 => unsafe {
                    io_apic = Some(IOAPIC {
                        ioapic_id: *ptr.add(2),
                        _reserved: *ptr.add(3),
                        ptr: (*ptr.add(4).cast::<u32>()) as *mut u8,
                        global_interrupt_base: *ptr.add(8).cast::<u32>(),
                    })
                },
                2 => unsafe {
                    io_apic_source_override = Some(IOAPICSourceOverride {
                        bus_source: *ptr.add(2),
                        irq_source: *ptr.add(3),
                        global_system_interrupt: *ptr.add(4).cast::<u32>(),
                        flags: *ptr.add(8).cast::<u16>(),
                    })
                },
                5 => lapic_ptr = unsafe { *(ptr.add(4).cast::<u64>()) } as *mut u8,
                _ => {}
            }

            ptr = unsafe { ptr.add((*ptr.add(1)) as usize) };
        }

        if io_apic.is_none() || io_apic_source_override.is_none() {
            return Err(());
        }

        let io_apic_ptr = io_apic.unwrap().ptr;

        crate::println!(
            "Found {} cores, IOAPIC {:p}, LAPIC {lapic_ptr:p}, Processor IDs:",
            cpus.len(),
            io_apic_ptr,
        );

        for apic in &cpus {
            crate::println!("    {}", apic.acpi_processor_id);
        }

        let apic = Self {
            io_apic: io_apic.unwrap(),
            local_apic: lapic_ptr,
            cpus: cpus.into(),
        };

        apic.write_lapic(0xF0, apic.read_lapic(0xF0) | 0x100);

        let io_apic_ver = apic.read_ioapic(0x01);

        let number_of_inputs = ((io_apic_ver >> 16) & 0xFF) + 1;

        crate::println!("{number_of_inputs}");

        // Take the keyboard vector table, then mask out the top most bit (interrupt mask), then,
        // mask out the bottom 8 bits, and put the kbd int in it setting the interrupt vector
        let keyboard_vt = ((apic.read_ioapic(0x12) & 0x7FFF) & 0xFF00) | 0x21;

        // enable keyboard interrupt
        apic.write_ioapic(0x12, keyboard_vt);

        return Ok(apic);
    }

    pub fn read_ioapic(&self, reg: u32) -> u32 {
        unsafe {
            core::ptr::write_volatile(self.io_apic.ptr.cast::<u32>(), reg & 0xff);
            return core::ptr::read_volatile(self.io_apic.ptr.cast::<u32>().add(4));
        }
    }

    pub fn write_ioapic(&self, reg: u32, value: u32) {
        unsafe {
            core::ptr::write_volatile(self.io_apic.ptr.cast::<u32>(), reg & 0xff);
            core::ptr::write_volatile(self.io_apic.ptr.cast::<u32>().add(4), value);
        }
    }

    pub fn read_lapic(&self, reg: u32) -> u32 {
        unsafe {
            return *self.local_apic.add(reg as usize).cast::<u32>();
        }
    }

    pub fn write_lapic(&self, reg: u32, value: u32) {
        unsafe {
            *self.local_apic.add(reg as usize).cast::<u32>() = value;
        }
    }

    pub fn end_of_interrupt(&self) {
        self.write_lapic(0xB0, 0x00);
    }
}

pub static APIC: Mutex<Option<APIC>> = Mutex::new(None);
