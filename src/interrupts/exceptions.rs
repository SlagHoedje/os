use flagset::{FlagSet, flags};
use interrupts::idt::InterruptStackFrame;
use panic::PanicType;
use x86_64::registers::control::Cr2;

flags! {
    enum PageFaultErrorCode: u32 {
        ProtectionViolation,
        Write,
        UserSpace,
        ReservedWrite,
        InstructionFetch,
    }
}

macro_rules! exception_handler {
    ($index:expr, $func:ident, $name:expr) => {
        pub extern "x86-interrupt" fn $func(stack_frame: &InterruptStackFrame) {
            crate::panic::panic(PanicType::KernelException{
                name: $name,
                index: $index,
                stack_frame,
                additional_info: None,
            })
        }
    };
}

macro_rules! exception_handler_error_code {
    ($index:expr, $func:ident, $name:expr) => {
        pub extern "x86-interrupt" fn $func(stack_frame: &InterruptStackFrame, error_code: u32) {
            crate::panic::panic(PanicType::KernelException{
                name: $name,
                index: $index,
                stack_frame,
                additional_info: Some(format_args!("\x1b[37mError Code: \x1b[97m{}", error_code)),
            })
        }
    };
}

exception_handler!(0x00, divide_by_zero_handler, "Divide-by-zero Error");
exception_handler!(0x01, debug_handler, "Debug");
exception_handler!(0x02, non_maskable_handler, "Non-maskable Interrupt");
exception_handler!(0x03, breakpoint_handler, "Breakpoint");
exception_handler!(0x04, overflow_handler, "Overflow");
exception_handler!(0x05, bound_range_handler, "Bound Range Exceeded");
exception_handler!(0x06, invalid_opcode_handler, "Invalid Opcode");
exception_handler!(0x07, device_not_available_handler, "Device Not Available");
exception_handler_error_code!(0x08, double_fault_handler, "Double Fault");
exception_handler_error_code!(0x0a, invalid_tss_handler, "Invalid TSS");
exception_handler_error_code!(0x0b, segment_not_present_handler, "Segment Not Present");
exception_handler_error_code!(0x0c, stack_segment_handler, "Stack-Segment Fault");
exception_handler_error_code!(0x0d, general_protection_handler, "General Protection Fault");
exception_handler!(0x10, x87_floating_point_handler, "x87 Floating-Point Exception");
exception_handler_error_code!(0x11, alignment_check_handler, "Alignment Check");
exception_handler!(0x12, machine_check_handler, "Machine Check");
exception_handler!(0x13, simd_floating_point_handler, "SIMD Floating-Point Exception");
exception_handler!(0x14, virtualization_handler, "Virtualization Exception");
exception_handler_error_code!(0x1e, security_handler, "Security Exception");

pub extern "x86-interrupt" fn page_fault_handler(stack_frame: &InterruptStackFrame, error_code: u32) {
    crate::panic::panic(PanicType::KernelException{
        name: "Page Fault",
        index: 0xe,
        stack_frame,
        additional_info: Some(format_args!(
            "\x1b[37mError Code: \x1b[97m{:#?}\n\x1b[37mAddress: \x1b[97m{:?}",
            FlagSet::<PageFaultErrorCode>::new_truncated(error_code),
            Cr2::read(),
        )),
    });
}