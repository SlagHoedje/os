use flagset::{flags, FlagSet};

flags! {
    pub enum RFlags: u64 {
        CPUID = 1 << 21,
        VirtualInterruptPending = 1 << 20,
        VirtualInterrupt = 1 << 19,
        AlignmentCheck = 1 << 18,
        Virtual8086Mode = 1 << 17,
        ResumeFlag = 1 << 16,
        NestedTask = 1 << 14,
        IOPLHigh = 1 << 13,
        IOPLLow = 1 << 12,
        OverflowFlag = 1 << 11,
        DirectionFlag = 1 << 10,
        InterruptFlag = 1 << 9,
        TrapFlag = 1 << 8,
        SignFlag = 1 << 7,
        ZeroFlag = 1 << 6,
        AuxiliaryCarryFlg = 1 << 4,
        ParityFlag = 1 << 2,
        CarryFlag = 1,
    }
}

impl RFlags {
    pub fn read() -> FlagSet<RFlags> {
        let raw: u64;
        unsafe { asm!("pushfq; pop $0" : "=r" (raw) :: "memory" : "intel") };
        FlagSet::new_truncated(raw)
    }
}