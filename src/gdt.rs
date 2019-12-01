use core::mem::size_of;

use bit_field::BitField;
use flagset::{flags, FlagSet};
use spin::Once;

use x86_64::instructions::tables::{DescriptorTablePointer, load_gdt, load_tss};
use x86_64::registers::segment::{CodeSegment, DataSegment};
use x86_64::VirtualAddress;

static TSS: Once<TaskStateSegment> = Once::new();
static GDT: Once<GlobalDescriptorTable> = Once::new();

flags! {
    enum DescriptorFlags: u64 {
        Privilege = 1 << 41,
        Conforming = 1 << 42,
        Executable = 1 << 43,
        UserSegment = 1 << 44,
        Present = 1 << 47,
        LongMode = 1 << 53,
    }
}

#[repr(C, packed)]
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

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    pub const fn new(index: u16, ring: u8) -> SegmentSelector {
        SegmentSelector(index << 3 | (ring as u16))
    }
}

pub struct GlobalDescriptorTable {
    table: [u64; 8],
    next_free: usize,
}

impl GlobalDescriptorTable {
    pub fn new() -> GlobalDescriptorTable {
        GlobalDescriptorTable {
            table: [0; 8],
            next_free: 1,
        }
    }

    pub fn pointer(&self) -> DescriptorTablePointer {
        DescriptorTablePointer::new(
            VirtualAddress::from_ptr(self.table.as_ptr()),
            (self.table.len() * size_of::<u64>() - 1) as u16,
        )
    }

    pub fn add_entry(&mut self, entry: Descriptor) -> SegmentSelector {
        let index = match entry {
            Descriptor::UserSegment(value) => self.push(value),
            Descriptor::SystemSegment(low, high) => {
                let index = self.push(low);
                self.push(high);
                index
            }
        };

        SegmentSelector::new(index as u16, 0)
    }

    fn push(&mut self, value: u64) -> usize {
        if self.next_free >= self.table.len() {
            panic!("GDT is full!");
        }

        let index = self.next_free;
        self.table[index] = value;
        self.next_free += 1;

        index
    }
}

pub enum Descriptor {
    UserSegment(u64),
    SystemSegment(u64, u64),
}

impl Descriptor {
    pub fn kernel_code_segment() -> Descriptor {
        let flags = DescriptorFlags::UserSegment | DescriptorFlags::Present |
            DescriptorFlags::Privilege | DescriptorFlags::Executable | DescriptorFlags::LongMode;

        Descriptor::UserSegment(flags.bits())
    }

    pub fn kernel_data_segment() -> Descriptor {
        let flags = DescriptorFlags::UserSegment | DescriptorFlags::Present |
            DescriptorFlags::Privilege | DescriptorFlags::LongMode;

        Descriptor::UserSegment(flags.bits())
    }

    pub fn tss_segment(tss: &'static TaskStateSegment) -> Descriptor {
        let ptr = tss as *const _ as u64;

        let mut low = FlagSet::from(DescriptorFlags::Present).bits();

        low.set_bits(16..40, ptr.get_bits(0..24));
        low.set_bits(56..64, ptr.get_bits(24..32));

        low.set_bits(0..16, (size_of::<TaskStateSegment>() - 1) as u64);
        low.set_bits(40..44, 0b1001);

        let mut high = 0;
        high.set_bits(0..32, ptr.get_bits(32..64));

        Descriptor::SystemSegment(low, high)
    }
}

pub fn init() {
    let mut code_selector = SegmentSelector(0);
    let mut data_selector = SegmentSelector(0);
    let mut tss_selector = SegmentSelector(0);

    let tss = TSS.call_once(|| {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[0] = {
            // TODO: Don't allocate this on the stack.
            const STACK_SIZE: usize = 4096;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = unsafe { STACK.as_mut_ptr() as u64 };
            let stack_end = stack_start + STACK_SIZE as u64;
            VirtualAddress::new(stack_end)
        };
        tss
    });

    let gdt = GDT.call_once(|| {
        let mut gdt = GlobalDescriptorTable::new();

        code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
        tss_selector = gdt.add_entry(Descriptor::tss_segment(tss));

        gdt
    });

    crate::kprintln!("Loading GDT...");
    load_gdt(gdt.pointer());

    crate::kprintln!("Loading segment selectors...");
    CodeSegment::write(code_selector);
    DataSegment::write(data_selector);

    unsafe {
        asm!("mov es, $0
              mov fs, $0
              mov gs, $0
              mov ss, $0" :: "r" (data_selector.0) : "memory" : "intel");
    }

    crate::kprintln!("Loading TSS...");
    load_tss(tss_selector);
}