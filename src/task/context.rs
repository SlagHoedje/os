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
        crate::kprintln!("{:?} -> {:?} | {:?}", next, self as *mut _, next as *const _);
        x86_64_context_switch(self as *mut _, next as *const _)
    }
}

#[naked]
extern "C" fn x86_64_context_switch(prev: *mut Context, next: *const Context) {
    // 'prev' -> rdi  'next' -> rsi (x86_64 C calling convention)
    // TODO: Remove unnecessary register saving due to C calling conventions (ex. rdi and rsi)
    unsafe {
        asm!("// Copy all registers into 'prev'
              pushfq
              pop qword ptr [rdi]
              mov [rdi + 0x08], r15
              mov [rdi + 0x10], r14
              mov [rdi + 0x18], r13
              mov [rdi + 0x20], r12
              mov [rdi + 0x28], r11
              mov [rdi + 0x30], r10
              mov [rdi + 0x38], r9
              mov [rdi + 0x40], r8
              mov [rdi + 0x48], rdi
              mov [rdi + 0x50], rsi
              mov [rdi + 0x58], rdx
              mov [rdi + 0x60], rcx
              mov [rdi + 0x68], rbx
              mov [rdi + 0x70], rax
              mov [rdi + 0x78], rbp

              // Switch around stack pointers
              mov [rdi + 0x80], rsp
              mov rsp, [rsi + 0x80]

              // Copy all registers from 'next'
              mov rbp, [rsi + 0x78]
              mov rax, [rsi + 0x70]
              mov rbx, [rsi + 0x68]
              mov rcx, [rsi + 0x60]
              mov rdx, [rsi + 0x58]
              // mov rsi, [rsi + 0x50]
              // mov rdi, [rsi + 0x48]
              mov r8, [rsi + 0x40]
              mov r9, [rsi + 0x38]
              mov r10, [rsi + 0x30]
              mov r11, [rsi + 0x28]
              mov r12, [rsi + 0x20]
              mov r13, [rsi + 0x18]
              mov r14, [rsi + 0x10]
              mov r15, [rsi + 0x08]
              push [rsi]
              popfq

              // Copy over rsi and rdi by pushing and popping the stack
              push [rsi + 0x50]
              push [rsi + 0x48]
              pop rdi
              pop rsi"
              :::: "intel", "volatile")
    }
}

extern "C" fn ret() {
    crate::kprintln!("process finished.");
    hlt_loop();
}