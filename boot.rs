#![feature(alloc_error_handler)]
#![feature(asm)]
#![feature(lang_items)]
#![no_std]

extern crate alloc;
extern crate rlibc;

use alloc::alloc::{alloc_zeroed, Layout};
use core::ptr::null_mut;
use core::ptr::{read_volatile, write_volatile};

// Bootloader function
fn boot_loader() {
    unsafe {
        asm!(
            // Set 16-bit mode
            "bits 16",

            // Set up the stack
            "mov ax, 0x7c00",
            "mov ss, ax",
            "mov sp, 0x1000",

            // Set up the data segment
            "mov ax, 0x0000",
            "mov ds, ax",
            "mov es, ax",

            // Read the kernel from the boot device
            "mov ah, 0x02",     // BIOS read sector function
            "mov al, 0x01",     // Number of sectors to read
            "mov ch, 0x00",     // Cylinder number
            "mov cl, 0x02",     // Sector number (1-based)
            "mov dh, 0x00",     // Head number
            "mov dl, 0x80",     // Boot device number
            "mov bx, 0x8000",   // Destination address in memory
            "int 0x13",         // Call BIOS interrupt

            // Switch to 64-bit mode
            "cli",                 // Disable interrupts
            "mov eax, cr0",
            "or eax, 0x80000001",  // Set the PE and PG bits
            "mov cr0, eax",
            "jmp CODE64_INIT",     // Jump to the 64-bit code

            // 64-bit initialization code
            BITS 64
            CODE64_INIT:
                "mov ax, 0x10", // Set up the data segment
                "mov ds, ax",
                "mov es, ax",
                "mov fs, ax",
                "mov gs, ax",
                "mov ss, ax",
                "mov rsp, 0x10000", // Set up the stack
                "call rust_munch", // Call rust_munch function to initialize memory and CPU

            // Padding and magic number to make the bootloader exactly 512 bytes
            "times 510-($-$$) db 0",
            "dw 0xaa55",

            // 64-bit kernel entry point
            "KERNEL_ENTRY:",
            // Your kernel code goes here
            "ret", // Return from the entry point
        );
    }
}

// Global allocator
#[global_allocator]
static ALLOCATOR: Allocator = Allocator::new();

// Implement GlobalAlloc for Allocator
unsafe impl alloc::alloc::GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Modify the allocation code to use the Allocator
        // ...
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Modify the deallocation code to use the Allocator
        // ...
    }
    // Constants
    const KERNEL_OFFSET: usize = 0xffffffff80000000;
    const PHYSICAL_OFFSET: usize = 0xffff_8000_0000_0000;
    const PT_ENTRIES: usize = 512;
    const PT_LEVELS: usize = 4;
    const PAGE_SIZE: usize = 4096;
    const KERNEL_BASE: usize = 0xffffffff_80000000;
    const KERNEL_SIZE: usize = 1 << 30; // 1GB
    const PAGE_FLAGS: u64 = 0x1 | 0x2 | 0x40; // Present, Writable, NX (no execute)
    
    // PageTableEntry struct
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    struct PageTableEntry(u64);



impl PageTableEntry {
    fn is_present(&self) -> bool {
        (self.0 & 1) != 0
    }
    fn frame_address(&self) -> usize {
        (self.0 & 0x000fffff_fffff000) as usize
    }

    fn set_frame_address(&mut self, addr: usize) {
        self.0 = (self.0 & 0xfff0000000000fff) | (addr as u64 & 0x000fffff_fffff000);
    }

    fn flags(&self) -> u64 {
        self.0 & 0xfff
    }

    fn set_flags(&mut self, flags: u64) {
        self.0 = (self.0 & 0xfffffffffffff000) | (flags & 0xfff);
    }
}

// PageTable struct
#[repr(align(4096))]
struct PageTable {
entries: [PageTableEntry; PT_ENTRIES],
}

impl PageTable {
    fn get_entry(&self, virtual_address: usize, level: usize) -> &PageTableEntry {
        let index = (virtual_address >> (level * 9 + 12)) & 0x1ff;
        &self.entries[index]
    }

    fn get_entry_mut(&mut self, virtual_address: usize, level: usize) -> &mut PageTableEntry {
        let index = (virtual_address >> (level * 9 + 12)) & 0x1ff;
        &mut self.entries[index]
    }

    fn get_next_level(&self, virtual_address: usize, level: usize) -> *const PageTable {
        let entry = self.get_entry(virtual_address, level);
        let table_address = entry.frame_address();
        (table_address as *const PageTable)
    }

    fn get_next_level_mut(&mut self, virtual_address: usize, level: usize) -> *mut PageTable {
        let entry = self.get_entry_mut(virtual_address, level);
        let table_address = entry.frame_address();
        (table_address as *mut PageTable)
    }

    fn map_page(&mut self, virtual_address: usize, physical_address: usize, flags: u64) {
        let mut table = self;
        for level in (0..PT_LEVELS - 1).rev() {
            let entry = table.get_entry_mut(virtual_address, level);
            if !entry.is_present() {
                let next_table = Box::leak(Box::new(PageTable::new()));
                entry.set_frame_address(next_table as *const _ as usize);
                entry.set_flags(flags | 0x1);
            }
            table = unsafe { &mut *table.get_next_level_mut(virtual_address, level) };
        }
        let entry = table.get_entry_mut(virtual_address, PT_LEVELS - 1);
        entry.set_frame_address(physical_address);
        entry.set_flags(flags | 0x1);
    }

    fn new() -> Self {
        PageTable {
            entries: [PageTableEntry(0); PT_ENTRIES],
        }
    }
}

// rust_munch function
#[no_mangle]
pub extern "C" fn rust_munch() -> ! {
unsafe {
        let mut cpuid_info: [u32; 4] = [0; 4];
        asm!("cpuid" : "={eax}"(cpuid_info[0]),
                       "={ebx}"(cpuid_info[1]), 
                       "={ecx}"(cpuid_info[2]), 
                       "={edx}"(cpuid_info[3]) : 
                       "{eax}"(0) : : "volatile");

                       let mut kern_mem = PageTable::new();

                       // Map kernel memory
                       for i in 0..(KERNEL_SIZE / PAGE_SIZE) {
                           let virtual_address = KERNEL_BASE + i * PAGE_SIZE;
                           let physical_address = virtual_address - KERNEL_OFFSET;
                           kern_mem.map_page(virtual_address, physical_address, PAGE_FLAGS);
                        }

         // Enable paging by setting the Paging Flag (PG) in the control register CR0
    asm!("mov %cr0, %rax ; or $$0x80000000, %rax ; mov %rax, %cr0");

    let mut cr4: usize;
    asm!("mov %cr4, $0" : "=r"(cr4) :: "memory" : "volatile");
    cr4 |= 1 << 5; // Enable global pages
    asm!("mov $0, %cr4" :: "r"(cr4) : "memory" : "volatile");

    // Jump to the kernel entry point
    let kernel_entry: extern "C" fn() -> ! = core::mem::transmute(KERNEL_BASE);
    kernel_entry();
}
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

// Panic handler
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
loop {}
}

// Allocator error handler
#[alloc_error_handler]
fn alloc_error_handler(_: core::alloc::Layout) -> ! {
loop {}
}
