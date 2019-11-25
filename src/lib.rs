#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(abi_x86_interrupt)]
#![feature(core_intrinsics)]
#![feature(ptr_internals)]
#![feature(const_fn)]
#![feature(asm)]
#![no_std]

extern crate alloc;
extern crate bit_field;
extern crate flagset;
extern crate lazy_static;
extern crate linked_list_allocator;
/// TODO: Replace with custom structure
extern crate multiboot2;
extern crate spin;
extern crate volatile;

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;

use linked_list_allocator::LockedHeap;

use fs::mount::MountFS;
use fs::ramdisk::Ramdisk;
use fs::vfs::{FileSystem, FileType, INode};
use memory::frame::AreaFrameAllocator;
use x86_64::PhysicalAddress;
use x86_64::registers::control::{Cr0, Cr0Flags};
use x86_64::registers::msr::{EFER, EFERFlags};

pub mod driver;
pub mod macros;
pub mod panic;
pub mod interrupts;
pub mod x86_64;
pub mod memory;
pub mod fs;

// TODO: Replace with custom implementation?
/// Global heap allocator. Used for allocating things on the heap, like Vec and Box.
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Kernel entry function. Called from assembly boot code
#[no_mangle]
pub extern "C" fn kmain(multiboot_information_address: usize) -> ! {
    driver::uart16550::UART.lock().init();
    driver::vga::WRITER.lock().clear_screen();

    kprintln!("\x1b[92m- \x1b[97mLoading interrupts...");
    interrupts::init();

    kprintln!("\x1b[92m- \x1b[97mLoading multiboot information structure...");
    let boot_info = unsafe { multiboot2::load(multiboot_information_address) };
    let memory_map_tag = boot_info.memory_map_tag()
        .expect("Memory map tag required");

    let elf_sections_tag = boot_info.elf_sections_tag()
        .expect("Elf-Sections tag required!");

    if let Some(name_tag) = boot_info.boot_loader_name_tag() {
        kprintln!("Bootloader: {}", name_tag.name());
    }

    let kernel_start = elf_sections_tag.sections().map(|s| s.start_address()).min().unwrap();
    let kernel_end = elf_sections_tag.sections().map(|s| s.end_address()).max().unwrap();

    let mut frame_allocator = AreaFrameAllocator::new(
        PhysicalAddress::new(kernel_start), PhysicalAddress::new(kernel_end),
        PhysicalAddress::new(boot_info.start_address() as u64),
        PhysicalAddress::new(boot_info.end_address() as u64),
        memory_map_tag.memory_areas()
    );

    kprintln!("\x1b[92m- \x1b[97mInitializing memory...");
    EFER::append(EFERFlags::NoExecuteEnable);
    Cr0::append(Cr0Flags::WriteProtect);
    let mut active_table = memory::paging::remap_kernel(&mut frame_allocator, &boot_info);

    kprintln!("Allocating heap...");
    memory::init_heap(&mut active_table, &mut frame_allocator);

    kprintln!("\x1b[92m- \x1b[97mTesting filesystem...");
    let ramdisk = Ramdisk::new();
    {
        let inode = ramdisk.root().create("hello.txt", FileType::File, 0o777)
            .expect("Error while creating inode for 'hello.txt'");
        inode.write_at(0, b"This is a file!").unwrap();
    }

    {
        let folder_inode = ramdisk.root().create("folder", FileType::Directory, 0o666)
            .expect("Error while creating inode for 'folder'");
        let inode = folder_inode.create("hello.txt", FileType::File, 0o777)
            .expect("Error while creating inode for 'hello.txt' 2");
        inode.write_at(0, b"This is another file").unwrap();
    }

    let root_ramdisk = Ramdisk::new();
    {
        root_ramdisk.root().create("tmp", FileType::Directory, 0o666).unwrap();
        let text_node = root_ramdisk.root().create("text.txt", FileType::File, 0o777).unwrap();
        text_node.write_at(0, b"test file").unwrap();
    }

    let root = MountFS::new(root_ramdisk.clone());
    root.root().find("tmp").unwrap().mount(ramdisk.clone()).unwrap();

    {
        let new_inode = root.root().find("text.txt").unwrap();

        let mut out = vec![0; 20];
        let bytes = new_inode.read_at(0, out.as_mut_slice()).unwrap();
        kprintln!("read {} bytes", bytes);
        kprintln!("{:?}", out);
        kprintln!("text.txt: {}", String::from_utf8(out).unwrap());
    }

    {
        let root_inode: Arc<dyn INode> = root.root();
        let new_inode = root_inode.resolve_follow("tmp/folder/hello.txt", 0).unwrap();

        let mut out = vec![0; 20];
        let bytes = new_inode.read_at(0, out.as_mut_slice()).unwrap();
        kprintln!("read {} bytes", bytes);
        kprintln!("{:?}", out);
        kprintln!("tmp/folder/hello.txt: {}", String::from_utf8(out).unwrap());
    }

    let root_inode: Arc<dyn INode> = root.root();
    kprintln!("files: {:?}", root_inode.list());
    kprintln!("files /tmp: {:?}", root_inode.find("tmp").unwrap().list());

    x86_64::instructions::hlt_loop()
}