use spin::Once;

use interrupts::idt::InterruptDescriptorTable;
use x86_64::instructions::tables::load_idt;
use x86_64::VirtualAddress;

pub mod idt;
pub mod exceptions;

static IDT: Once<InterruptDescriptorTable> = Once::new();

macro_rules! push_registers {
    () => {
        asm!("push rax
              push rbx
              push rcx
              push rdx
              push rsi
              push rdi
              push r8
              push r9
              push r10
              push r11
              push r12
              push r13
              push r14
              push r15
              push rbp"
             :::: "intel", "volatile");
    };
}

macro_rules! pop_registers {
    () => {
        asm!("pop rbp
              pop r15
              pop r14
              pop r13
              pop r12
              pop r11
              pop r10
              pop r9
              pop r8
              pop rdi
              pop rsi
              pop rdx
              pop rcx
              pop rbx
              pop rax"
             :::: "intel", "volatile");
    };
}

#[macro_export]
macro_rules! idt_handler {
    ($kind: expr, $name: ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm!("push 0
                      push $0" :: "i" ($kind) :: "intel");
                push_registers!();
                asm!("mov rdi, rsp
                      call $0"
                      :: "i" ($name as extern "C" fn(&StackFrame))
                      : "rdi" : "intel");
                pop_registers!();
                asm!("add rsp, 16
                      iretq" :::: "intel", "volatile");

                core::intrinsics::unreachable()
            }
        }

        wrapper
    }};
}

#[macro_export]
macro_rules! idt_handler_error_code {
    ($kind: expr, $name: ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm!("push $0" :: "i" ($kind) :: "intel");
                push_registers!();
                asm!("mov rdi, rsp
                      call $0"
                      :: "i" ($name as extern "C" fn(&StackFrame))
                      : "rdi" : "intel");
                pop_registers!();
                asm!("add rsp, 16
                      iretq" :::: "intel", "volatile");

                core::intrinsics::unreachable()
            }
        }

        wrapper
    }};
}

#[repr(C)]
#[derive(Debug)]
pub struct StackFrame {
    pub rbp: u64,
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
    pub kind: u64,
    pub error_code: u64,
    pub instruction_pointer: VirtualAddress,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: VirtualAddress,
    pub stack_segment: u64,
}

pub fn init() {
    let idt = IDT.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();
        use interrupts::exceptions::*;
        idt.set_handler(0x00, idt_handler!(0x00, divide_by_zero_handler));
        idt.set_handler(0x01, idt_handler!(0x01, debug_handler));
        idt.set_handler(0x02, idt_handler!(0x02, non_maskable_handler));
        idt.set_handler(0x03, idt_handler!(0x03, breakpoint_handler));
        idt.set_handler(0x04, idt_handler!(0x04, overflow_handler));
        idt.set_handler(0x05, idt_handler!(0x05, bound_range_handler));
        idt.set_handler(0x06, idt_handler!(0x06, invalid_opcode_handler));
        idt.set_handler(0x07, idt_handler!(0x07, device_not_available_handler));
        idt.set_handler(0x08, idt_handler_error_code!(0x08, double_fault_handler)).set_stack_index(0);
        idt.set_handler(0x0a, idt_handler_error_code!(0x0a, invalid_tss_handler));
        idt.set_handler(0x0b, idt_handler_error_code!(0x0b, segment_not_present_handler));
        idt.set_handler(0x0c, idt_handler_error_code!(0x0c, stack_segment_handler));
        idt.set_handler(0x0d, idt_handler_error_code!(0x0d, general_protection_handler));
        idt.set_handler(0x0e, idt_handler_error_code!(0x0e, page_fault_handler));
        idt.set_handler(0x10, idt_handler!(0x10, x87_floating_point_handler));
        idt.set_handler(0x11, idt_handler_error_code!(0x11, alignment_check_handler));
        idt.set_handler(0x12, idt_handler!(0x12, machine_check_handler));
        idt.set_handler(0x13, idt_handler!(0x13, simd_floating_point_handler));
        idt.set_handler(0x14, idt_handler!(0x14, virtualization_handler));
        idt.set_handler(0x1e, idt_handler_error_code!(0x1e, security_handler));
        idt
    });

    crate::kprintln!("Loading IDT...");
    load_idt(idt.pointer());
}