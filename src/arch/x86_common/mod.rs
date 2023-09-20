pub mod io;
pub mod pic;

#[repr(u8)]
pub enum MTRRMode {
    Uncacheable = 0x00,
    WriteCombining = 0x01,
    Writethrough = 0x04,
    WriteProtect = 0x05,
    Writeback = 0x06,
}

const IA32_MTRR_PHYSBASE0: u32 = 0x200;
const IA32_MTRR_PHYSMASK0: u32 = 0x201;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub unsafe fn set_mtrr(base: u64, size: u64, mode: MTRRMode) {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::__cpuid_count;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::__cpuid_count;

    unsafe {
        let cpu_id = __cpuid_count(1, 0);

        let mtrr_supported = (cpu_id.eax & (1 << 12)) != 0;

        if mtrr_supported == false {
            return;
        }
    }

    // Calculate the mask that corresponds to the size.
    let mask = !((size - 1) | 0xFFF); // Assumes a 4KB page size.

    // Set the Write-Combined memory type (0x02).
    let memory_type = mode as u8;

    // Use inline assembly to write to the MSR registers.
    unsafe {
        core::arch::asm!(
                "wrmsr",
                in("ecx") IA32_MTRR_PHYSBASE0,
                in("eax") (base & 0xFFFFFFFF) | memory_type as u64,
                in("edx") ((base >> 32) & 0xFFFFFFFF),
        );

        core::arch::asm!(
                "wrmsr",
                in("ecx") IA32_MTRR_PHYSMASK0,
                in("eax") (mask & 0xFFFFFFFF),
                in("edx") ((mask >> 32) & 0xFFFFFFFF),
        );
    }
}
