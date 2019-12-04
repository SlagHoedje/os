use x86_64::registers::rflags::RFlags;

#[inline]
pub fn enable() {
    unsafe {
        asm!("sti" :::: "volatile");
    }
}

#[inline]
pub fn disable() {
    unsafe {
        asm!("cli" :::: "volatile");
    }
}

pub fn with_disabled<T>(f: impl Fn() -> T) -> T {
    disable();
    let out = f();
    enable();

    out
}

pub fn are_enabled() -> bool {
    RFlags::read().contains(RFlags::InterruptFlag)
}