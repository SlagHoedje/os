use core::mem::size_of;

use bit_field::BitField;
use flagset::{flags, FlagSet};

use interrupts::tss::TaskStateSegment;
use x86_64::instructions::tables::DescriptorTablePointer;
use x86_64::VirtualAddress;

flags! {
    enum DescriptorFlags: u64 {
        Conforming = 1 << 42,
        Executable = 1 << 43,
        UserSegment = 1 << 44,
        Present = 1 << 47,
        LongMode = 1 << 53,
    }
}

#[derive(Copy, Clone)]
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
            DescriptorFlags::Executable | DescriptorFlags::LongMode;

        Descriptor::UserSegment(flags.bits())
    }

    pub fn kernel_data_segment() -> Descriptor {
        let flags = DescriptorFlags::UserSegment | DescriptorFlags::Present |
            DescriptorFlags::LongMode;

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