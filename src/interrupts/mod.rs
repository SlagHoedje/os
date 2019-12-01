use spin::Once;

use interrupts::idt::InterruptDescriptorTable;
use x86_64::instructions::tables::load_idt;

mod idt;

static IDT: Once<InterruptDescriptorTable> = Once::new();

pub fn init() {
    let idt = IDT.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();
        idt.set_handler(3, breakpoint_handler);
        idt
    });

    crate::kprintln!("Loading IDT...");
    load_idt(idt.pointer());
}

pub extern "C" fn breakpoint_handler() -> ! {
    panic!("breakpoint exception")
}