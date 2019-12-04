use core::ops::{Deref, DerefMut};

use flagset::FlagSet;
use multiboot2::{BootInformation, ElfSectionFlags};

use memory::frame::{Frame, FrameAllocator};
use memory::PAGE_SIZE;
use memory::paging::entry::EntryFlags;
use memory::paging::mapper::Mapper;
use memory::paging::temporary_page::TemporaryPage;
use x86_64::{PhysicalAddress, VirtualAddress};
use x86_64::instructions::TLB;
use x86_64::registers::control::Cr3;

pub mod entry;
pub mod table;
pub mod mapper;

mod temporary_page;

const TABLE_ENTRY_COUNT: usize = 512;

pub struct ActivePageTable {
    mapper: Mapper,
}

impl ActivePageTable {
    unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            mapper: Mapper::new(),
        }
    }

    pub fn with<F>(&mut self, table: &mut InactivePageTable, temporary_page: &mut TemporaryPage, f: F) where F: FnOnce(&mut Mapper) {
        {
            let backup = Frame::containing_address(Cr3::read());

            let p4_table = temporary_page.map_table_frame(Frame(backup.0), self);

            self.p4_mut()[511].set(Frame(table.p4_frame.0), EntryFlags::Present | EntryFlags::Writable);
            TLB::flush_all();

            f(self);

            p4_table[511].set(backup, EntryFlags::Present | EntryFlags::Writable);
            TLB::flush_all();
        }

        temporary_page.unmap(self);
    }

    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        let old_table = InactivePageTable {
            p4_frame: Frame::containing_address(Cr3::read()),
        };

        Cr3::write(new_table.p4_frame.start_address());

        old_table
    }
}

impl Deref for ActivePageTable {
    type Target=  Mapper;

    fn deref(&self) -> &Mapper {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Mapper {
        &mut self.mapper
    }
}

pub struct InactivePageTable {
    p4_frame: Frame,
}

impl InactivePageTable {
    pub fn new(frame: Frame, active_table: &mut ActivePageTable, temporary_page: &mut TemporaryPage) -> InactivePageTable {
        {
            let table = temporary_page.map_table_frame(Frame(frame.0), active_table);
            table.zero();
            table[511].set(Frame(frame.0), EntryFlags::Present | EntryFlags::Writable);
        }

        temporary_page.unmap(active_table);

        InactivePageTable {
            p4_frame: frame,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Page(pub usize);

impl Page {
    pub fn containing_address(address: VirtualAddress) -> Page {
        assert!(address.as_u64() < 0x0000_8000_0000_0000 ||
                    address.as_u64() >= 0xffff_8000_0000_0000,
                "Invalid address: {:?}", address);
        Page((address.as_u64() as usize) / PAGE_SIZE)
    }

    pub fn start_address(self) -> VirtualAddress {
        VirtualAddress::new((self.0 * PAGE_SIZE) as u64)
    }

    fn p4_index(self) -> usize {
        (self.0 >> 27) & 0o777
    }

    fn p3_index(self) -> usize {
        (self.0 >> 18) & 0o777
    }

    fn p2_index(self) -> usize {
        (self.0 >> 9) & 0o777
    }

    fn p1_index(self) -> usize {
        self.0 & 0o777
    }

    pub fn range_inclusive(start: Page, end: Page) -> PageIter {
        PageIter {
            start,
            end,
        }
    }
}

#[derive(Clone)]
pub struct PageIter {
    start: Page,
    end: Page,
}

impl Iterator for PageIter {
    type Item = Page;

    fn next(&mut self) -> Option<Page> {
        if self.start.0 <= self.end.0 {
            let page = self.start;
            self.start.0 += 1;
            Some(page)
        } else {
            None
        }
    }
}
pub fn remap_kernel<A>(allocator: &mut A, boot_info: &BootInformation) -> ActivePageTable where A: FrameAllocator {
    let mut temporary_page = TemporaryPage::new(Page(0xcafe_babe), allocator);

    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = {
        let frame = allocator.allocate_frame().expect("No more frames!");
        InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
    };

    active_table.with(&mut new_table, &mut temporary_page, |mapper| {
        let elf_sections_tag = boot_info.elf_sections_tag()
            .expect("Memory map tag required!");

        for section in elf_sections_tag.sections() {
            if !section.is_allocated() {
                continue;
            }

            assert_eq!(section.start_address() % PAGE_SIZE as u64, 0);

            let mut flags = FlagSet::new_truncated(0);

            if section.flags().contains(ElfSectionFlags::ALLOCATED) {
                flags |= EntryFlags::Present
            }

            if section.flags().contains(ElfSectionFlags::WRITABLE) {
                flags |= EntryFlags::Writable;
            }

            if !section.flags().contains(ElfSectionFlags::EXECUTABLE) {
                flags |= EntryFlags::NoExecute;
            }

            let start_frame = Frame::containing_address(PhysicalAddress::new(section.start_address()));
            let end_frame = Frame::containing_address(PhysicalAddress::new(section.end_address() - 1));

            for frame in Frame::range_inclusive(start_frame, end_frame) {
                mapper.identity_map(frame, flags, allocator);
            }
        }

        let vga_buffer_frame = Frame::containing_address(PhysicalAddress::new(0xb8000));
        mapper.identity_map(vga_buffer_frame, EntryFlags::Writable, allocator);

        let multiboot_start = Frame::containing_address(
            PhysicalAddress::new(boot_info.start_address() as u64)
        );

        let multiboot_end = Frame::containing_address(
            PhysicalAddress::new(boot_info.end_address() as u64 - 1)
        );

        for frame in Frame::range_inclusive(multiboot_start, multiboot_end) {
            mapper.identity_map(frame, EntryFlags::Present, allocator);
        }
    });

    crate::kprintln!("Switching to new page table...");
    let old_table = active_table.switch(new_table);

    let old_p4_page = Page::containing_address(
        VirtualAddress::new(old_table.p4_frame.start_address().as_u64())
    );

    active_table.unmap(old_p4_page, allocator);
    crate::kprintln!("Created kernel stack guard page at {:?}", old_p4_page.start_address());

    active_table
}