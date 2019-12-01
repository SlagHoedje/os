use spin::Once;

use interrupts::idt::InterruptDescriptorTable;
use x86_64::instructions::tables::load_idt;
use x86_64::VirtualAddress;

mod idt;

static IDT: Once<InterruptDescriptorTable> = Once::new();

#[macro_export]
macro_rules! idt_handler {
    ($kind: expr, $name: ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm!("push 0
                      push $1
                      mov rdi, rsp
                      sub rsp, 8
                      call $0"
                      :: "i" ($name as extern "C" fn(&StackFrame) -> !), "i" ($kind)
                      : "rdi" : "intel");

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
                asm!("push $1
                      mov rdi, rsp
                      sub rsp, 8
                      call $0"
                      :: "i" ($name as extern "C" fn(&StackFrame) -> !), "i" ($kind)
                      : "rdi" : "intel");

                core::intrinsics::unreachable()
            }
        }

        wrapper
    }};
}

#[repr(C)]
#[derive(Debug)]
pub struct StackFrame {
    kind: u64,
    error_code: u64,
    instruction_pointer: VirtualAddress,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: VirtualAddress,
    stack_segment: u64,
}

pub fn init() {
    let idt = IDT.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();
        idt.set_handler(3, idt_handler!(3, breakpoint_handler));
        idt.set_handler(14, idt_handler_error_code!(14, page_fault_handler));
        idt
    });

    crate::kprintln!("Loading IDT...");
    load_idt(idt.pointer());
}

pub extern "C" fn breakpoint_handler(stack_frame: &StackFrame) -> ! {
    panic!("breakpoint exception. \n{:#?}", stack_frame)
}

pub extern "C" fn page_fault_handler(stack_frame: &StackFrame) -> ! {
    panic!("page fault. \n{:#?}", stack_frame)
}