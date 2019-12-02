use x86_64::VirtualAddress;
use x86_64::registers::control::Cr3;

pub mod tables;
pub mod interrupts;

pub struct TLB;

impl TLB {
    pub fn flush(addr: VirtualAddress) {
        unsafe { asm!("invlpg [$0]" :: "r" (addr.as_u64()) : "memory" : "intel") };
    }

    pub fn flush_all() {
        Cr3::write(Cr3::read());
    }
}

pub fn hlt_loop() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}