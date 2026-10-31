use crate::mmu::map::{SYS_L1_PAGE_TABLE, SYS_MMU};
use crate::mmu::*;
use arbitrary_int::u4;

#[derive(Debug)]
struct SectionDescMap {
    start: usize,
    size_mib: u16,
    attr: MapAttributes,
}

const ATTR_RAM: MapAttributes = MapAttributes {
    non_secure: false,
    non_global: false,
    shareable: true,
    access: FullAccess,
    memory_attrs: CacheableMemory {
        inner: WriteBackWriteAlloc,
        outer: WriteBackWriteAlloc,
    },
    domain: u4::new(0),
    execute_never: false,
};

const ATTR_DEVICE: MapAttributes = MapAttributes {
    non_secure: false,
    non_global: false,
    shareable: false,
    access: FullAccess,
    memory_attrs: ShareableDevice,
    domain: u4::new(0),
    execute_never: false,
};

const INITIAL_MAP: [SectionDescMap; 5] = [
    //
    // SectionDescs for PS
    //
    SectionDescMap {
        start: 0x0000_0000,
        size_mib: 1024,
        attr: ATTR_RAM,
    },
    SectionDescMap {
        start: 0xE000_0000,
        size_mib: 16,
        attr: ATTR_DEVICE,
    },
    SectionDescMap {
        start: 0xF800_0000,
        size_mib: 16,
        attr: ATTR_DEVICE,
    },
    //
    // SectionDescs for PL
    //
    SectionDescMap {
        start: 0x4120_0000,
        size_mib: 1,
        attr: ATTR_DEVICE,
    },
    SectionDescMap {
        start: 0x42C0_0000,
        size_mib: 1,
        attr: ATTR_DEVICE,
    },
];

pub fn init() {
    if aarch32_cpu::asm::core_id() == 0 {
        build_initial_l1_table(&SYS_L1_PAGE_TABLE);
    }

    let ttbr0 = aarch32_cpu::register::Ttbr0::new_with_raw_value(0)
        .with_address(core::ptr::addr_of!(SYS_L1_PAGE_TABLE) as usize)
        .with_irgn(false)
        .with_nos(false)
        .with_rgn(aarch32_cpu::register::ttbr0::Region::WriteBackWriteAllocCacheable)
        .with_s(true)
        .with_c(true);
    unsafe { aarch32_cpu::register::Ttbr0::write(ttbr0) }
    crate::mmu::asm::enable_mmu_and_cache();

    if aarch32_cpu::asm::core_id() == 0 {
        crate::mmu::page_table_pool::init();
        critical_section::with(|cs| {
            SYS_MMU.borrow(cs).replace(Some(&SYS_L1_PAGE_TABLE));
        });
    }
}

fn build_initial_l1_table(table: &L1Table<MAX_L1_ENTRIES>) {
    for i in 0..INITIAL_MAP.len() {
        let addr = INITIAL_MAP[i].start;
        let size = INITIAL_MAP[i].size_mib as usize * SECTION_SIZE;
        if let Err(e) = table.map(addr, addr, size, &INITIAL_MAP[i].attr, false) {
            panic!("{:?}", e);
        }
    }
}
