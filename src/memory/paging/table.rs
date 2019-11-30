use core::ops::{Index, IndexMut};

use memory::paging::entry::{Entry, EntryFlags};
use memory::paging::TABLE_ENTRY_COUNT;
use x86_64::VirtualAddress;
use core::marker::PhantomData;
use memory::frame::FrameAllocator;

#[allow(clippy::inconsistent_digit_grouping)]
pub const P4: *mut PageTable<Level4> = 0o177777_777_777_777_777_0000 as *mut _;

pub struct Level4;
pub struct Level3;
pub struct Level2;
pub struct Level1;

pub trait TableLevel {}
impl TableLevel for Level4 {}
impl TableLevel for Level3 {}
impl TableLevel for Level2 {}
impl TableLevel for Level1 {}

pub trait HierarchicalLevel: TableLevel { type NextLevel: TableLevel; }
impl HierarchicalLevel for Level4 { type NextLevel = Level3; }
impl HierarchicalLevel for Level3 { type NextLevel = Level2; }
impl HierarchicalLevel for Level2 { type NextLevel = Level1; }

pub struct PageTable<L: TableLevel> {
    entries: [Entry; TABLE_ENTRY_COUNT],
    _phantom: PhantomData<L>
}

impl<L: TableLevel> PageTable<L> {
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }
}

impl<L: HierarchicalLevel> PageTable<L> {
    pub fn next_table(&self, index: usize) -> Option<&PageTable<L::NextLevel>> {
        self.next_table_address(index)
            .map(|address| unsafe { &*address.as_ptr() })
    }

    pub fn next_table_mut(&self, index: usize) -> Option<&mut PageTable<L::NextLevel>> {
        self.next_table_address(index)
            .map(|address| unsafe { &mut *address.as_mut_ptr() })
    }

    pub fn next_table_create<A>(&mut self, index: usize, allocator: &mut A) -> &mut PageTable<L::NextLevel>
        where A: FrameAllocator {
        if self.next_table(index).is_none() {
            assert!(!self.entries[index].flags().contains(EntryFlags::HugePage));

            let frame = allocator.allocate_frame().expect("No frames available!");
            self.entries[index].set(frame, EntryFlags::Present | EntryFlags::Writable);
            self.next_table_mut(index).unwrap().zero();
        }

        self.next_table_mut(index).unwrap()
    }

    fn next_table_address(&self, index: usize) -> Option<VirtualAddress> {
        let entry_flags = self[index].flags();

        if entry_flags.contains(EntryFlags::Present) &&
            !entry_flags.contains(EntryFlags::HugePage) {
            let table_address = self as *const _ as u64;
            Some(VirtualAddress::new((table_address << 9) | ((index as u64) << 12)))
        } else {
            None
        }
    }
}

impl<L: TableLevel> Index<usize> for PageTable<L> {
    type Output = Entry;

    fn index(&self, index: usize) -> &Entry {
        &self.entries[index]
    }
}

impl<L: TableLevel> IndexMut<usize> for PageTable<L> {
    fn index_mut(&mut self, index: usize) -> &mut Entry {
        &mut self.entries[index]
    }
}