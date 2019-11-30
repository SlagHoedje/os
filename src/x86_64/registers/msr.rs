use flagset::{flags, FlagSet};

flags! {
    pub enum EFERFlags: u64 {
        SystemCallExtensions = 1,
        LongModeEnable = 1 << 8,
        LongModeActive = 1 << 10,
        NoExecuteEnable = 1 << 11,
        SecureVirtualMachineEnable = 1 << 12,
        LongModeSegmentLimitEnable = 1 << 13,
        Fastfxsavefxrstor = 1 << 14,
        TranslationCacheExtension = 1 << 15,
    }
}

pub struct MSR;

impl MSR {
    pub fn read(reg: u64) -> u64 {
        let low: u32;
        let high: u32;

        unsafe {
            asm!("rdmsr" : "={eax}" (low), "={edx}" (high) : "{ecx}" (reg) : "memory" : "volatile");
        }

        ((high as u64) << 32) | (low as u64)
    }

    pub fn write(reg: u64, value: u64) {
        let low = value as u32;
        let high = (value >> 32) as u32;

        unsafe {
            asm!("wrmsr" :: "{ecx}" (reg), "{eax}" (low), "{edx}" (high) : "memory" : "volatile");
        }
    }
}

pub struct EFER;

impl EFER {
    pub const MSR_REG: u64 = 0xc000_0080;

    pub fn read() -> FlagSet<EFERFlags> {
        FlagSet::new_truncated(MSR::read(EFER::MSR_REG))
    }

    pub fn write(flags: impl Into<FlagSet<EFERFlags>>) {
        let old_value = MSR::read(EFER::MSR_REG);
        let reserved = old_value & !(FlagSet::<EFERFlags>::full().bits());
        let new_value = reserved | flags.into().bits();

        MSR::write(EFER::MSR_REG, new_value);
    }

    pub fn append(flags: impl Into<FlagSet<EFERFlags>>) {
        let old_value = MSR::read(EFER::MSR_REG);
        let new_value = old_value | flags.into().bits();

        MSR::write(EFER::MSR_REG, new_value);
    }
}