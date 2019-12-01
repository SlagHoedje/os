use alloc::alloc::Layout;
use core::fmt;
use core::panic::PanicInfo;

use x86_64::VirtualAddress;

use crate::kprintln;
use interrupts::StackFrame;

/// An enum to indicate what kind of panic has occurred. This is used in conjunction with the
/// `panic::panic` function.
pub enum PanicType<'a> {
    /// Used for internal panicking. This is the type of panic that is most used, simply because
    /// this is the wrapper for the rust `panic!` macro, which also is used by the rust core
    /// library.
    KernelAssert(&'a PanicInfo<'a>),

    /// Used when a CPU exception occurs. Takes a lot of additional arguments to accurately display
    /// debugging info, because these exceptions are the hardest to debug. This is only used by the
    /// IDT exception handlers.
    KernelException { name: &'a str, stack_frame: &'a StackFrame, additional_info: Option<fmt::Arguments<'a>> },

    /// Used when an allocation error occurs. This mostly happens due to running out of memory in
    /// the heap.
    AllocationError(Layout)
}

/// Panic and halt the kernel. Will print all available debugging information to the console.
pub fn panic(panic: PanicType) -> ! {
    unsafe { crate::driver::vga::WRITER.force_unlock() };
    kprintln!("\n\x1b[31m!!! \x1b[91mKERNEL PANIC");

    match panic {
        PanicType::KernelAssert(info) => {
            let message = info.message().copied()
                .unwrap_or_else(|| format_args!("No message."));
            kprintln!("\x1b[37m// \x1b[97m{}", message);

            if let Some(location) = info.location() {
                kprintln!("\n\x1b[91mat {}", location);
            }
        },
        PanicType::KernelException { name, stack_frame, additional_info } => {
            kprintln!("\x1b[37m// \x1b[97mCPU EXCEPTION: '{}' (IDX: 0x{:02.x})", name, stack_frame.kind);

            kprintln!("\n\x1b[91mStack Frame:");

            // TODO: Fix padding
            kprintln!("\x1b[37mInstruction Pointer: \x1b[97m{:_<12?}\x1b[37m  Code Segment: \x1b[97m{}", stack_frame.instruction_pointer, stack_frame.code_segment);
            kprintln!("\x1b[37mStack Pointer: \x1b[97m{:_<12?}\x1b[37m        Stack Segment: \x1b[97m{}", stack_frame.stack_pointer, stack_frame.stack_segment);
            kprintln!("\x1b[37mCPU Flags: \x1b[97m0x{:x}", stack_frame.cpu_flags);
            kprintln!();
            kprintln!("\x1b[37mRAX: \x1b[97m0x{: <16x}  \x1b[37mRDI: \x1b[97m0x{: <16x}  \x1b[37mR12: \x1b[97m0x{: <16x}", stack_frame.rax, stack_frame.rdi, stack_frame.r12);
            kprintln!("\x1b[37mRBX: \x1b[97m0x{: <16x}  \x1b[37mR8:  \x1b[97m0x{: <16x}  \x1b[37mR13: \x1b[97m0x{: <16x}", stack_frame.rbx, stack_frame.r8, stack_frame.r13);
            kprintln!("\x1b[37mRCX: \x1b[97m0x{: <16x}  \x1b[37mR9:  \x1b[97m0x{: <16x}  \x1b[37mR14: \x1b[97m0x{: <16x}", stack_frame.rcx, stack_frame.r9, stack_frame.r14);
            kprintln!("\x1b[37mRDX: \x1b[97m0x{: <16x}  \x1b[37mR10: \x1b[97m0x{: <16x}  \x1b[37mR15: \x1b[97m0x{: <16x}", stack_frame.rdx, stack_frame.r10, stack_frame.r15);
            kprintln!("\x1b[37mRSI: \x1b[97m0x{: <16x}  \x1b[37mR11: \x1b[97m0x{: <16x}  \x1b[37mRBP: \x1b[97m0x{: <16x}", stack_frame.rsi, stack_frame.r11, stack_frame.rbp);


            if let Some(info) = additional_info {
                kprintln!("\n\x1b[91mAdditional Info:");
                kprintln!("{}", info);
            }

            let test = 0;
            kprintln!("\nStack variable pointer: {:?}", VirtualAddress::from_ptr(&test));
        },
        PanicType::AllocationError(layout) => {
            kprintln!("\x1b[37m// \x1b[97mAllocation error: {:?}", layout);
        }
    }

    crate::x86_64::instructions::hlt_loop()
}

/// Default Rust panic handler. Calls `panic::panic` internally.
#[cfg(not(test))]
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    panic(PanicType::KernelAssert(info))
}

/// Default Rust allocation error handler. Calls `panic::panic` internally.
#[cfg(not(test))]
#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic(PanicType::AllocationError(layout))
}