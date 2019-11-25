use core::marker::PhantomData;
use core::mem::size_of;

use bit_field::BitField;

use x86_64::instructions::tables::DescriptorTablePointer;
use x86_64::registers::segment::CodeSegment;
use x86_64::VirtualAddress;
use interrupts::gdt::SegmentSelector;

pub type HandlerFn = extern "x86-interrupt" fn(&InterruptStackFrame);
pub type HandlerFnErrorCode = extern "x86-interrupt" fn(&InterruptStackFrame, u32);

#[repr(C)]
#[repr(align(16))]
pub struct InterruptDescriptorTable {
    pub divide_by_zero: Entry<HandlerFn>,
    pub debug: Entry<HandlerFn>,
    pub non_maskable: Entry<HandlerFn>,
    pub breakpoint: Entry<HandlerFn>,
    pub overflow: Entry<HandlerFn>,
    pub bound_range: Entry<HandlerFn>,
    pub invalid_opcode: Entry<HandlerFn>,
    pub device_not_available: Entry<HandlerFn>,
    pub double_fault: Entry<HandlerFnErrorCode>,
    pub coprocessor_segment_overrun: Entry<HandlerFn>,
    pub invalid_tss: Entry<HandlerFnErrorCode>,
    pub segment_not_present: Entry<HandlerFnErrorCode>,
    pub stack_segment: Entry<HandlerFnErrorCode>,
    pub general_protection: Entry<HandlerFnErrorCode>,
    pub page_fault: Entry<HandlerFnErrorCode>,
    pub reserved0: Entry<HandlerFn>,
    pub x87_floating_point: Entry<HandlerFn>,
    pub alignment_check: Entry<HandlerFnErrorCode>,
    pub machine_check: Entry<HandlerFn>,
    pub simd_floating_point: Entry<HandlerFn>,
    pub virtualization: Entry<HandlerFn>,
    pub reserved1: Entry<HandlerFn>,
    pub reserved2: Entry<HandlerFn>,
    pub reserved3: Entry<HandlerFn>,
    pub reserved4: Entry<HandlerFn>,
    pub reserved5: Entry<HandlerFn>,
    pub reserved6: Entry<HandlerFn>,
    pub reserved7: Entry<HandlerFn>,
    pub reserved8: Entry<HandlerFn>,
    pub reserved9: Entry<HandlerFn>,
    pub security: Entry<HandlerFnErrorCode>,
    pub reserved10: Entry<HandlerFn>,
}

impl InterruptDescriptorTable {
    pub const fn new() -> InterruptDescriptorTable {
        InterruptDescriptorTable {
            divide_by_zero: Entry::missing(),
            debug: Entry::missing(),
            non_maskable: Entry::missing(),
            breakpoint: Entry::missing(),
            overflow: Entry::missing(),
            bound_range: Entry::missing(),
            invalid_opcode: Entry::missing(),
            device_not_available: Entry::missing(),
            double_fault: Entry::missing(),
            coprocessor_segment_overrun: Entry::missing(),
            invalid_tss: Entry::missing(),
            segment_not_present: Entry::missing(),
            stack_segment: Entry::missing(),
            general_protection: Entry::missing(),
            page_fault: Entry::missing(),
            reserved0: Entry::missing(),
            x87_floating_point: Entry::missing(),
            alignment_check: Entry::missing(),
            machine_check: Entry::missing(),
            simd_floating_point: Entry::missing(),
            virtualization: Entry::missing(),
            reserved1: Entry::missing(),
            reserved2: Entry::missing(),
            reserved3: Entry::missing(),
            reserved4: Entry::missing(),
            reserved5: Entry::missing(),
            reserved6: Entry::missing(),
            reserved7: Entry::missing(),
            reserved8: Entry::missing(),
            reserved9: Entry::missing(),
            security: Entry::missing(),
            reserved10: Entry::missing(),
        }
    }

    pub fn pointer(&self) -> DescriptorTablePointer {
        DescriptorTablePointer::new(
            VirtualAddress::from_ptr(self as *const _),
            (size_of::<InterruptDescriptorTable>() - 1) as u16
        )
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Entry<F> {
    pointer_low: u16,
    gdt_selector: SegmentSelector,
    options: EntryOptions,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
    phantom: PhantomData<F>,
}

impl<F> Entry<F> {
    pub const fn missing() -> Entry<F> {
        Entry {
            pointer_low: 0,
            pointer_middle: 0,
            pointer_high: 0,
            gdt_selector: SegmentSelector(0),
            options: EntryOptions::minimal(),
            reserved: 0,
            phantom: PhantomData,
        }
    }

    fn handler_addr(&mut self, handler: VirtualAddress) -> EntryOptions {
        let handler_ptr = handler.as_u64();

        self.pointer_low = handler_ptr as u16;
        self.pointer_middle = (handler_ptr >> 16) as u16;
        self.pointer_high = (handler_ptr >> 32) as u32;

        self.gdt_selector = CodeSegment::read();

        self.options.set_present(true);

        self.options
    }
}

macro_rules! impl_handler_fn {
    ($type:ty) => {
        impl Entry<$type> {
            pub fn handler_fn(&mut self, handler: $type) -> EntryOptions {
                self.handler_addr(VirtualAddress::new(handler as u64))
            }
        }
    };
}

impl_handler_fn!(HandlerFn);
impl_handler_fn!(HandlerFnErrorCode);

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct EntryOptions(u16);

impl EntryOptions {
    pub const fn minimal() -> EntryOptions {
        EntryOptions(0b0000_1110_0000_0000)
    }

    pub fn set_present(&mut self, present: bool) {
        self.0.set_bit(15, present);
    }

    pub fn set_stack_index(&mut self, index: u8) {
        self.0.set_bits(0..2, (index + 1) as u16);
    }
}

#[repr(C)]
pub struct InterruptStackFrame {
    pub instruction_pointer: VirtualAddress,
    pub code_segment: u64,
    pub rflags: u64,
    pub stack_pointer: VirtualAddress,
    pub stack_segment: u64,
}