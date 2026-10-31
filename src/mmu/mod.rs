mod asm;
mod init;
mod map;
mod page_table_pool;

pub use aarch32_cpu::mmu::AccessPermissions;
pub use aarch32_cpu::mmu::AccessPermissions::*;
pub use aarch32_cpu::mmu::CachePolicy::*;
pub use aarch32_cpu::mmu::L1EntryType;
pub use aarch32_cpu::mmu::MemoryRegionAttributes;
pub use aarch32_cpu::mmu::MemoryRegionAttributes::*;

pub use init::init;
pub(crate) use map::{L1Table, PageTable};
pub use map::{MapAttributes, MmuError, map, remap};

const NUM_POOL_ELEMENTS: usize = 256;

const MAX_L1_ENTRIES: usize = 4096;
const NUM_L2_ENTRIES: usize = 256;

pub const SECTION_SIZE: usize = 1024 * 1024;
pub const SUPER_SECTION_SIZE: usize = 16 * SECTION_SIZE;
pub const PAGE_SIZE: usize = 4 * 1024;
pub const LARGE_PAGE_SIZE: usize = 64 * 1024;

pub unsafe fn debug() {
    let entries = unsafe { &*map::SYS_L1_PAGE_TABLE.entries.get() };

    for e in entries.iter().enumerate() {
        crate::println!("{:03X}: {:?}", e.0, e.1);
    }
}
