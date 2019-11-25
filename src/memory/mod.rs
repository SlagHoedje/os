use memory::frame::FrameAllocator;
use memory::paging::{ActivePageTable, Page};
use memory::paging::entry::EntryFlags;
use x86_64::VirtualAddress;

pub mod frame;
pub mod paging;

pub const PAGE_SIZE: usize = 4096;

pub const HEAP_START: VirtualAddress = VirtualAddress::new(0x4444_4444_0000);
pub const HEAP_SIZE: usize = 100 * 1024;

pub fn init_heap<A>(active_table: &mut ActivePageTable, allocator: &mut A) where A: FrameAllocator {
    let page_range = {
        let heap_end = VirtualAddress::new(HEAP_START.as_u64() + HEAP_SIZE as u64);
        let heap_start_page = Page::containing_address(HEAP_START);
        let heap_end_page = Page::containing_address(heap_end);

        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    let flags = EntryFlags::Present | EntryFlags::Writable;

    for page in page_range {
        active_table.map(page, flags, allocator);
    }

    unsafe {
        crate::ALLOCATOR.lock().init(HEAP_START.as_u64() as usize, HEAP_SIZE);
    }
}