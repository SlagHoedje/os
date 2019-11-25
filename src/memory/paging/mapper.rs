use core::ptr::Unique;

use flagset::FlagSet;

use memory::frame::{Frame, FrameAllocator};
use memory::PAGE_SIZE;
use memory::paging::{Page, TABLE_ENTRY_COUNT};
use memory::paging::entry::EntryFlags;
use memory::paging::table::{Level4, P4, PageTable};
use x86_64::{PhysicalAddress, VirtualAddress};
use x86_64::instructions::TLB;

pub struct Mapper {
    p4: Unique<PageTable<Level4>>,
}

impl Mapper {
    pub unsafe fn new() -> Mapper {
        Mapper {
            p4: Unique::new_unchecked(P4),
        }
    }

    pub fn map<A>(&mut self, page: Page, flags: impl Into<FlagSet<EntryFlags>>, allocator: &mut A) where A: FrameAllocator {
        let frame = allocator.allocate_frame().expect("Out of memory!");
        self.map_to(page, frame, flags, allocator)
    }

    pub fn identity_map<A>(&mut self, frame: Frame, flags: impl Into<FlagSet<EntryFlags>>, allocator: &mut A) where A: FrameAllocator {
        let page = Page::containing_address(
            VirtualAddress::new(frame.start_address().as_u64())
        );

        self.map_to(page, frame, flags, allocator);
    }

    pub fn map_to<A>(&mut self, page: Page, frame: Frame, flags: impl Into<FlagSet<EntryFlags>>, allocator: &mut A) where A: FrameAllocator {
        let p3 = self.p4_mut().next_table_create(page.p4_index(), allocator);
        let p2 = p3.next_table_create(page.p3_index(), allocator);
        let p1 = p2.next_table_create(page.p2_index(), allocator);

        assert!(p1[page.p1_index()].is_unused());
        p1[page.p1_index()].set(frame, flags.into() | EntryFlags::Present);
    }

    pub fn unmap<A>(&mut self, page: Page, allocator: &mut A) where A: FrameAllocator {
        assert!(self.translate(page.start_address()).is_some());

        let p1 = self.p4_mut().next_table_mut(page.p4_index())
            .and_then(|p3| p3.next_table_mut(page.p3_index()))
            .and_then(|p2| p2.next_table_mut(page.p2_index()))
            .expect("Huge pages are not supported!");

        let frame = p1[page.p1_index()].pointed_frame().unwrap();
        p1[page.p1_index()].set_unused();

        TLB::flush(page.start_address());

        // TODO: Unmap p1 p2 p3 if empty.
        allocator.deallocate_frame(frame);
    }

    pub fn translate(&self, address: VirtualAddress) -> Option<PhysicalAddress> {
        let offset = address.as_u64() % PAGE_SIZE as u64;
        self.translate_page(Page::containing_address(address))
            .map(|frame| PhysicalAddress::new((frame.0 * PAGE_SIZE) as u64 + offset))
    }

    pub fn translate_page(&self, page: Page) -> Option<Frame> {
        let p3 = self.p4().next_table(page.p4_index());

        let huge_page = || {
            p3.and_then(|p3| {
                let p3_entry = &p3[page.p3_index()];

                if let Some(start_frame) = p3_entry.pointed_frame() {
                    if p3_entry.flags().contains(EntryFlags::HugePage) {
                        assert_eq!(start_frame.0 % (TABLE_ENTRY_COUNT.pow(2)), 0);

                        return Some(Frame(
                            start_frame.0 + page.p2_index() * TABLE_ENTRY_COUNT + page.p1_index()
                        ));
                    }
                }

                if let Some(p2) = p3.next_table(page.p3_index()) {
                    let p2_entry = &p2[page.p2_index()];

                    if let Some(start_frame) = p2_entry.pointed_frame() {
                        if p2_entry.flags().contains(EntryFlags::HugePage) {
                            assert_eq!(start_frame.0 % TABLE_ENTRY_COUNT, 0);

                            return Some(Frame(start_frame.0 + page.p1_index()));
                        }
                    }
                }

                None
            })
        };

        p3.and_then(|p3| p3.next_table(page.p3_index()))
            .and_then(|p2| p2.next_table(page.p2_index()))
            .and_then(|p1| p1[page.p1_index()].pointed_frame())
            .or_else(huge_page)
    }

    pub fn p4(&self) -> &PageTable<Level4> {
        unsafe { self.p4.as_ref() }
    }

    pub fn p4_mut(&mut self) -> &mut PageTable<Level4> {
        unsafe { self.p4.as_mut() }
    }
}