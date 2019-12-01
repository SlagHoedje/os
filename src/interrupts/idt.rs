use bit_field::BitField;
use x86_64::instructions::tables::DescriptorTablePointer;
use core::mem::size_of;
use x86_64::VirtualAddress;

pub type HandlerFn = extern "C" fn() -> !;

#[repr(transparent)]
pub struct InterruptDescriptorTable([Entry; 16]);

impl InterruptDescriptorTable {
    pub fn new() -> InterruptDescriptorTable {
        InterruptDescriptorTable([Entry::missing(); 16])
    }

    pub fn set_handler(&mut self, entry: usize, handler: HandlerFn) -> &mut EntryOptions {
        // TODO: Get the proper code selector
        self.0[entry] = Entry::new(8, handler);
        &mut self.0[entry].options
    }

    pub fn pointer(&'static self) -> DescriptorTablePointer {
        DescriptorTablePointer::new(
            VirtualAddress::from_ptr(self as *const _),
            (size_of::<InterruptDescriptorTable>() - 1) as u16,
        )
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Entry {
    pointer_low: u16,
    gdt_selector: u16,
    options: EntryOptions,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
}

impl Entry {
    fn missing() -> Entry {
        Entry {
            gdt_selector: 0,
            pointer_low: 0,
            pointer_middle: 0,
            pointer_high: 0,
            options: EntryOptions::minimal(),
            reserved: 0,
        }
    }

    fn new(gdt_selector: u16, handler: HandlerFn) -> Entry {
        let pointer = handler as u64;

        Entry {
            gdt_selector,
            pointer_low: pointer as u16,
            pointer_middle: (pointer >> 16) as u16,
            pointer_high: (pointer >> 32) as u32,
            options: EntryOptions::new(),
            reserved: 0,
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct EntryOptions(u16);

impl EntryOptions {
    fn minimal() -> EntryOptions {
        let mut options = 0;
        options.set_bits(9..12, 0b111);
        EntryOptions(options)
    }

    fn new() -> EntryOptions {
        let mut options = EntryOptions::minimal();
        options.set_present(true).disable_interrupts(true);
        options
    }

    pub fn set_present(&mut self, present: bool) -> &mut EntryOptions {
        self.0.set_bit(15, present);
        self
    }

    pub fn disable_interrupts(&mut self, disable: bool) -> &mut EntryOptions {
        self.0.set_bit(8, !disable);
        self
    }

    pub fn set_privilege_level(&mut self, dpl: u16) -> &mut EntryOptions {
        self.0.set_bits(13..15, dpl);
        self
    }

    pub fn set_stack_index(&mut self, index: u16) -> &mut EntryOptions {
        self.0.set_bits(0..3, index);
        self
    }
}