use memory::paging::{PageIter, ActivePageTable, Page};
use memory::frame::FrameAllocator;
use memory::{Stack, PAGE_SIZE};
use memory::paging::entry::EntryFlags;

pub struct StackAllocator {
    range: PageIter,
}

impl StackAllocator {
    pub fn new(page_range: PageIter) -> StackAllocator {
        StackAllocator {
            range: page_range,
        }
    }

    pub fn alloc_stack<A: FrameAllocator>(&mut self, active_table: &mut ActivePageTable, frame_allocator: &mut A, size_in_pages: usize) -> Option<Stack> {
        if size_in_pages == 0 {
            return None;
        }

        let mut range = self.range.clone();

        let guard_page = range.next();
        let stack_start = range.next();
        let stack_end = if size_in_pages == 1 {
            stack_start
        } else {
            range.nth(size_in_pages - 2)
        };

        match (guard_page, stack_start, stack_end) {
            (Some(_), Some(start), Some(end)) => {
                self.range = range;

                for page in Page::range_inclusive(start, end) {
                    active_table.map(page, EntryFlags::Writable, frame_allocator);
                }

                let top_of_stack = end.start_address() + PAGE_SIZE as u64;
                Some(Stack { top: top_of_stack, bottom: start.start_address() })
            },
            _ => None
        }
    }
}