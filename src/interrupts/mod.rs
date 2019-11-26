use spin::Once;

use interrupts::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use interrupts::idt::InterruptDescriptorTable;
use interrupts::tss::TaskStateSegment;
use x86_64::registers::segment::CodeSegment;
use x86_64::VirtualAddress;

pub mod idt;
pub mod tss;
pub mod gdt;
pub mod exceptions;

static IDT: Once<InterruptDescriptorTable> = Once::new();
static TSS: Once<TaskStateSegment> = Once::new();
static GDT: Once<GlobalDescriptorTable> = Once::new();

pub fn init() {
    let tss = TSS.call_once(|| {
        let mut tss = TaskStateSegment::new();

        tss.interrupt_stack_table[0] = {
            const STACK_SIZE: usize = 4096;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = unsafe { &STACK } as *const _ as u64;
            let stack_end = stack_start + STACK_SIZE as u64;

            VirtualAddress::new(stack_end)
        };

        tss
    });

    let mut code_selector = SegmentSelector(0);
    let mut tss_selector = SegmentSelector(0);

    let gdt = GDT.call_once(|| {
        let mut gdt = GlobalDescriptorTable::new();

        code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        gdt.add_entry(Descriptor::kernel_data_segment());
        tss_selector = gdt.add_entry(Descriptor::tss_segment(tss));

        gdt
    });

    crate::kprintln!("Loading GDT...");
    crate::x86_64::instructions::tables::load_gdt(gdt.pointer());
    CodeSegment::write(code_selector);

    crate::kprintln!("Loading TSS...");
    crate::x86_64::instructions::tables::load_tss(tss_selector); // TODO: Stack Overflow still doesn't work

    let idt = IDT.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();

        idt.divide_by_zero.handler_fn(exceptions::divide_by_zero_handler);
        idt.debug.handler_fn(exceptions::debug_handler);
        idt.non_maskable.handler_fn(exceptions::non_maskable_handler);
        idt.breakpoint.handler_fn(exceptions::breakpoint_handler);
        idt.overflow.handler_fn(exceptions::overflow_handler);
        idt.bound_range.handler_fn(exceptions::bound_range_handler);
        idt.invalid_opcode.handler_fn(exceptions::invalid_opcode_handler);
        idt.device_not_available.handler_fn(exceptions::device_not_available_handler);
        idt.double_fault.handler_fn(exceptions::double_fault_handler).set_stack_index(0);
        idt.invalid_tss.handler_fn(exceptions::invalid_tss_handler);
        idt.segment_not_present.handler_fn(exceptions::segment_not_present_handler);
        idt.stack_segment.handler_fn(exceptions::stack_segment_handler);
        idt.general_protection.handler_fn(exceptions::general_protection_handler);
        idt.page_fault.handler_fn(exceptions::page_fault_handler);
        idt.x87_floating_point.handler_fn(exceptions::x87_floating_point_handler);
        idt.alignment_check.handler_fn(exceptions::alignment_check_handler);
        idt.machine_check.handler_fn(exceptions::machine_check_handler);
        idt.simd_floating_point.handler_fn(exceptions::simd_floating_point_handler);
        idt.virtualization.handler_fn(exceptions::virtualization_handler);
        idt.security.handler_fn(exceptions::security_handler);

        idt
    });

    crate::kprintln!("Loading IDT...");
    crate::x86_64::instructions::tables::load_idt(idt.pointer());
}