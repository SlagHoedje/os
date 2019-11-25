use core::marker::PhantomData;

pub trait PortValue {}
impl PortValue for u8 {}
impl PortValue for u16 {}
impl PortValue for u32 {}

pub struct Port<T: PortValue> {
    port: u16,
    phantom: PhantomData<T>,
}

impl<T: PortValue> Port<T> {
    pub const fn new(port: u16) -> Port<T> {
        Port {
            port,
            phantom: PhantomData,
        }
    }
}

impl Port<u8> {
    pub fn read(&self) -> u8 {
        let value: u8;
        unsafe {
            asm!("inb %dx, %al" : "={al}" (value) : "{dx}" (self.port) :: "volatile");
        }
        value
    }

    pub fn write(&self, value: u8) {
        unsafe {
            asm!("outb %al, %dx" :: "{dx}" (self.port), "{al}" (value) :: "volatile")
        }
    }
}

impl Port<u16> {
    pub fn read(&self) -> u16 {
        let value: u16;
        unsafe {
            asm!("inw %dx, %ax" : "={ax}" (value) : "{dx}" (self.port) :: "volatile");
        }
        value
    }

    pub fn write(&self, value: u16) {
        unsafe {
            asm!("outw %ax, %dx" :: "{dx}" (self.port), "{ax}" (value) :: "volatile")
        }
    }
}

impl Port<u32> {
    pub fn read(&self) -> u32 {
        let value: u32;
        unsafe {
            asm!("inb %dx, %eax" : "={eax}" (value) : "{dx}" (self.port) :: "volatile");
        }
        value
    }

    pub fn write(&self, value: u32) {
        unsafe {
            asm!("outb %eax, %dx" :: "{eax}" (self.port), "{al}" (value) :: "volatile")
        }
    }
}