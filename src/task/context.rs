use core::mem::size_of;

use x86_64::instructions::hlt_loop;
use x86_64::VirtualAddress;

/// A struct that contains all registers that need to be saved for a context switch.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Context {
    rflags: u64,
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rdi: u64,
    rsi: u64,
    rdx: u64,
    rcx: u64,
    rbx: u64,
    rax: u64,
    rbp: u64,
    rsp: VirtualAddress,
}

impl Context {
    /// Creates a new 'Context' where all registers are set to zero.
    pub const fn empty() -> Context {
        Context {
            rflags: 0,
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            r11: 0,
            r10: 0,
            r9: 0,
            r8: 0,
            rdi: 0,
            rsi: 0,
            rdx: 0,
            rcx: 0,
            rbx: 0,
            rax: 0,
            rbp: 0,
            rsp: VirtualAddress::new(0),
        }
    }

    /// Cretes a new 'Context' with the specified stack and the interrupt flag set.
    pub fn new(stack_top: VirtualAddress, proc_entry: u64) -> Context {
        let mut ctx = Context {
            rflags: 0,//FlagSet::from(RFlags::InterruptFlag).bits(),
            rbp: stack_top.as_u64(),
            rsp: stack_top,
            ..Context::empty()
        };

        crate::kprintln!("ctx made");

        unsafe {
            ctx.push_stack(ret as u64);
            ctx.push_stack(proc_entry);
        }

        crate::kprintln!("stack pushed");

        ctx
    }

    /// Pushes an item to the stack of this context
    ///
    /// # Safety
    /// The process this context is attached to needs to expect this change, else it will probably
    /// crash the corresponding process.
    pub unsafe fn push_stack(&mut self, item: u64) {
        self.rsp -= size_of::<u64>() as u64;
        *self.rsp.as_mut_ptr() = item;
    }

    /// Switch from this context to another context, saving all registers in this context.
    #[inline]
    pub fn switch_to(&mut self, next: &Context) {
        crate::kprintln!("{:?}", next);
        x86_64_context_switch(self as *mut _, next as *const _)
    }
}

#[naked]
extern "C" fn x86_64_context_switch(prev: *mut Context, next: *const Context) {
    // TODO: Don't use r14 and r15
    // TODO: Remove unnecessary register saving due to C calling conventions
    unsafe {
        asm!("pushfq
              pop qword ptr [$0]
              mov [$0 + 0x08], r15
              mov [$0 + 0x10], r14
              mov [$0 + 0x18], r13
              mov [$0 + 0x20], r12
              mov [$0 + 0x28], r11
              mov [$0 + 0x30], r10
              mov [$0 + 0x38], r9
              mov [$0 + 0x40], r8
              mov [$0 + 0x48], rdi
              mov [$0 + 0x50], rsi
              mov [$0 + 0x58], rdx
              mov [$0 + 0x60], rcx
              mov [$0 + 0x68], rbx
              mov [$0 + 0x70], rax
              mov [$0 + 0x78], rbp

              mov [$0 + 0x80], rsp
              mov rsp, [$1 + 0x80]

              mov rbp, [$1 + 0x78]
              mov rax, [$1 + 0x70]
              mov rbx, [$1 + 0x68]
              mov rcx, [$1 + 0x60]
              mov rdx, [$1 + 0x58]
              mov rsi, [$1 + 0x50]
              mov rdi, [$1 + 0x48]
              mov r8, [$1 + 0x40]
              mov r9, [$1 + 0x38]
              mov r10, [$1 + 0x30]
              mov r11, [$1 + 0x28]
              mov r12, [$1 + 0x20]
              mov r13, [$1 + 0x18]
              mov r14, [$1 + 0x10]
              //mov r15, [$1 + 0x08]
              push [$1]
              popfq

              push [$1 + 0x08]
              pop r15"
              :: "{r14}" (prev), "{r15}" (next) :: "intel", "volatile")
    }
}

extern "C" fn ret() {
    crate::kprintln!("process finished.");
    hlt_loop();
}