use flagset::{flags, FlagSet};

use memory::frame::Frame;
use x86_64::PhysicalAddress;

flags! {
    pub enum EntryFlags: u64 {
        Present = 1,
        Writable = 1 << 1,
        UserAccessible = 1 << 2,
        WriteThrough = 1 << 3,
        NoCache = 1 << 4,
        Accessed = 1 << 5,
        Dirty = 1 << 6,
        HugePage = 1 << 7,
        Global = 1 << 8,
        NoExecute = 1 << 63,
    }
}

pub struct Entry(u64);

impl Entry {
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }

    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    pub fn flags(&self) -> FlagSet<EntryFlags> {
        FlagSet::new_truncated(self.0)
    }

    pub fn pointed_frame(&self) -> Option<Frame> {
        if self.flags().contains(EntryFlags::Present) {
            Some(Frame::containing_address(
                PhysicalAddress::new(self.0 & 0x000fffff_fffff000)
            ))
        } else {
            None
        }
    }

    pub fn set(&mut self, frame: Frame, flags: impl Into<FlagSet<EntryFlags>>) {
        assert_eq!(frame.start_address().as_u64() & !0x000fffff_fffff000, 0);
        self.0 = (frame.start_address().as_u64()) | flags.into().bits();
    }
}