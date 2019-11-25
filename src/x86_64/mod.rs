use core::fmt;
use core::fmt::{Formatter, Error};

pub mod instructions;
pub mod registers;
pub mod port;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct VirtualAddress(u64);

impl VirtualAddress {
    pub const fn new(address: u64) -> VirtualAddress {
        VirtualAddress(address)
    }

    pub const fn null() -> VirtualAddress {
        VirtualAddress(0)
    }

    pub fn from_ptr<T>(ptr: *const T) -> VirtualAddress {
        VirtualAddress(ptr as u64)
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }

    pub fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }

    pub fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }
}

impl From<VirtualAddress> for u64 {
    fn from(address: VirtualAddress) -> u64 {
        address.0
    }
}

impl fmt::Debug for VirtualAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_fmt(format_args!("V:{:#x}", self.0))
    }
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct PhysicalAddress(u64);

impl PhysicalAddress {
    pub fn new(address: u64) -> PhysicalAddress {
        PhysicalAddress(address)
    }

    pub fn as_u64(self) -> u64 {
    self.0
}
}

impl From<PhysicalAddress> for u64 {
    fn from(address: PhysicalAddress) -> u64 {
        address.0
    }
}

impl fmt::Debug for PhysicalAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_fmt(format_args!("P:{:#x}", self.0))
    }
}