use flagset::{flags, FlagSet};

use x86_64::{PhysicalAddress, VirtualAddress};

flags! {
    pub enum Cr0Flags: u64 {
        ProtectedModeEnable = 1,
        MonitorCoprocessor = 1 << 1,
        EmulateCoprocessor = 1 << 2,
        TaskSwitched = 1 << 3,
        NumericError = 1 << 5,
        WriteProtect = 1 << 16,
        AlignmentMask = 1 << 18,
        NotWriteThrough = 1 << 29,
        CacheDisable = 1 << 30,
        Paging = 1 << 31,
    }
}

pub struct Cr0;

impl Cr0 {
    pub fn read() -> FlagSet<Cr0Flags> {
        FlagSet::new_truncated(Cr0::read_raw())
    }

    pub fn write(flags: impl Into<FlagSet<Cr0Flags>>) {
        let old_value = Cr0::read_raw();
        let reserved = old_value & !(FlagSet::<Cr0Flags>::full().bits());
        let new_value = reserved | flags.into().bits();

        Cr0::write_raw(new_value);
    }

    pub fn append(flags: impl Into<FlagSet<Cr0Flags>>) {
        let old_value = Cr0::read_raw();
        let new_value = old_value | flags.into().bits();

        Cr0::write_raw(new_value);
    }

    fn read_raw() -> u64 {
        let value: u64;
        unsafe { asm!("mov $0, cr0" : "=r" (value) ::: "intel") };
        value
    }

    fn write_raw(value: u64) {
        unsafe { asm!("mov cr0, $0" :: "r" (value) : "memory" : "intel") }
    }
}

pub struct Cr2;

impl Cr2 {
    pub fn read() -> VirtualAddress {
        let out: VirtualAddress;
        unsafe { asm!("mov $0, cr2" : "=r" (out) ::: "intel") };
        out
    }
}

pub struct Cr3;

impl Cr3 {
    pub fn read() -> PhysicalAddress {
        let out: PhysicalAddress;
        unsafe { asm!("mov $0, cr3" : "=r" (out) ::: "intel") };
        out
    }

    pub fn write(address: PhysicalAddress) {
        unsafe { asm!("mov cr3, $0" :: "r" (address.0) : "memory" : "intel") }
    }
}