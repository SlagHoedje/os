use x86_64::VirtualAddress;
use interrupts::gdt::SegmentSelector;

#[repr(C, packed)]
pub struct DescriptorTablePointer {
    limit: u16,
    base: VirtualAddress,
}

impl DescriptorTablePointer {
    pub fn new(base: VirtualAddress, limit: u16) -> DescriptorTablePointer {
        DescriptorTablePointer {
            base,
            limit,
        }
    }
}

pub fn load_idt(ptr: DescriptorTablePointer) {
    unsafe { asm!("lidt [$0]" :: "r" (&ptr) : "memory" : "intel") };
}

pub fn load_gdt(ptr: DescriptorTablePointer) {
    unsafe { asm!("lgdt [$0]" :: "r" (&ptr) : "memory" : "intel") };
}

pub fn load_tss(selector: SegmentSelector) {
    unsafe { asm!("ltr $0" :: "r" (selector.0)) };
}