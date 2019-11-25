use x86_64::VirtualAddress;

pub struct TaskStateSegment {
    pub reserved0: u32,
    pub privilege_stack_table: [VirtualAddress; 3],
    pub reserved1: u64,
    pub interrupt_stack_table: [VirtualAddress; 7],
    pub reserved2: u64,
    pub reserved3: u16,
    pub iomap_base: u16,
}

impl TaskStateSegment {
    pub const fn new() -> TaskStateSegment {
        TaskStateSegment {
            privilege_stack_table: [VirtualAddress::null(); 3],
            interrupt_stack_table: [VirtualAddress::null(); 7],
            iomap_base: 0,
            reserved0: 0,
            reserved1: 0,
            reserved2: 0,
            reserved3: 0,
        }
    }
}