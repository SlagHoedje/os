use multiboot2::{MemoryArea, MemoryAreaIter};

use memory::PAGE_SIZE;
use x86_64::PhysicalAddress;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame(pub usize);

impl Frame {
    pub fn containing_address(address: PhysicalAddress) -> Frame {
        Frame((address.as_u64() as usize) / PAGE_SIZE)
    }

    pub fn start_address(&self) -> PhysicalAddress {
        PhysicalAddress::new((self.0 * PAGE_SIZE) as u64)
    }

    pub fn range_inclusive(start: Frame, end: Frame) -> FrameIter {
        FrameIter {
            start,
            end,
        }
    }
}

pub struct FrameIter {
    start: Frame,
    end: Frame,
}

impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        if self.start <= self.end {
            let frame = Frame(self.start.0);
            self.start.0 += 1;
            Some(frame)
        } else {
            None
        }
    }
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}

pub struct AreaFrameAllocator<'a> {
    next_free_frame: Frame,
    current_area: Option<&'a MemoryArea>,
    areas: MemoryAreaIter<'a>,
    kernel_start: Frame,
    kernel_end: Frame,
    multiboot_start: Frame,
    multiboot_end: Frame,
}

impl<'a> AreaFrameAllocator<'a> {
    pub fn new(kernel_start: PhysicalAddress, kernel_end: PhysicalAddress,
               multiboot_start: PhysicalAddress, multiboot_end: PhysicalAddress,
               memory_areas: MemoryAreaIter<'a>) -> AreaFrameAllocator {
        let mut allocator = AreaFrameAllocator {
            next_free_frame: Frame::containing_address(PhysicalAddress::new(0)),
            current_area: None,
            areas: memory_areas,
            kernel_start: Frame::containing_address(kernel_start),
            kernel_end: Frame::containing_address(kernel_end),
            multiboot_start: Frame::containing_address(multiboot_start),
            multiboot_end: Frame::containing_address(multiboot_end)
        };

        allocator.choose_next_area();
        allocator
    }

    fn choose_next_area(&mut self) {
        self.current_area = self.areas.clone().filter(|area| {
            let address = area.end_address() - 1;
            Frame::containing_address(PhysicalAddress::new(address)) >= self.next_free_frame
        }).min_by_key(|area| area.start_address());

        if let Some(area) = self.current_area {
            let start_frame = Frame::containing_address(PhysicalAddress::new(area.start_address()));
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}

impl<'a> FrameAllocator for AreaFrameAllocator<'a> {
    fn allocate_frame(&mut self) -> Option<Frame> {
        if let Some(area) = self.current_area {
            let frame = Frame(self.next_free_frame.0);

            let current_area_last_frame = {
                let address = PhysicalAddress::new(area.end_address() - 1);
                Frame::containing_address(address)
            };

            if frame > current_area_last_frame {
                self.choose_next_area();
            } else if frame >= self.kernel_start && frame <= self.kernel_end {
                self.next_free_frame = Frame(self.kernel_end.0 + 1);
            } else if frame >= self.multiboot_start && frame <= self.multiboot_end {
                self.next_free_frame = Frame(self.multiboot_end.0 + 1);
            } else {
                self.next_free_frame.0 += 1;
                return Some(frame);
            }

            self.allocate_frame()
        } else {
            None
        }
    }

    fn deallocate_frame(&mut self, _frame: Frame) {
        // TODO: unimplemented!()
    }
}