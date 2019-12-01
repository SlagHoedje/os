use memory::frame::FrameAllocator;
use memory::paging::{ActivePageTable, Page};
use memory::paging::entry::EntryFlags;
use x86_64::VirtualAddress;
use core::alloc::{Layout, GlobalAlloc};

pub mod frame;
pub mod paging;

pub const PAGE_SIZE: usize = 4096;

pub const HEAP_START: VirtualAddress = VirtualAddress::new(0x4444_4444_0000);
pub const HEAP_SIZE: usize = 100 * 1024;

/// A struct that represents a memory stack for a program or the kernel.
pub struct Stack {
    top: VirtualAddress,
    bottom: VirtualAddress,
}

impl Stack {
    pub fn top(&self) -> VirtualAddress {
        self.top
    }

    pub fn bottom(&self) -> VirtualAddress {
        self.bottom
    }
}

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

pub fn alloc_stack(size_in_pages: usize) -> Option<Stack> {
    let ptr = unsafe { crate::ALLOCATOR.alloc(
        Layout::array::<u8>(PAGE_SIZE * size_in_pages).ok()?
    ) };

    let bottom = VirtualAddress::from_ptr(ptr);
    let top = VirtualAddress::new(ptr as u64 + (PAGE_SIZE * size_in_pages) as u64);
    Some(Stack { top, bottom })
}