use aarch32_cpu::mmu::AccessPermissions::*;
use aarch32_cpu::mmu::CachePolicy::*;
use aarch32_cpu::mmu::MemoryRegionAttributes::*;
use aarch32_cpu::mmu::{L1Section, L1Table, NUM_L1_PAGE_TABLE_ENTRIES, SectionAttributes};
use arbitrary_int::{u4, u12};

static SYS_L1_PAGE_TABLE: L1Table = build_initial_l1_table();

const SECTION_ATTR_RAM: SectionAttributes = SectionAttributes {
    non_global: false,
    p_bit: false,
    shareable: true,
    access: FullAccess,
    memory_attrs: CacheableMemory {
        inner: WriteBackWriteAlloc,
        outer: WriteBackWriteAlloc,
    }
    .as_raw(),
    domain: u4::new(0),
    execute_never: false,
};

const SECTION_ATTR_DEVICE: SectionAttributes = SectionAttributes {
    non_global: false,
    p_bit: false,
    shareable: false,
    access: FullAccess,
    memory_attrs: ShareableDevice.as_raw(),
    domain: u4::new(0),
    execute_never: false,
};

struct SectionMap {
    start: usize,
    size_mib: u16,
    attr: SectionAttributes,
}

const INITIAL_MAP: [SectionMap; 5] = [
    //
    // Sections for PS
    //
    SectionMap {
        start: 0x0000_0000,
        size_mib: 1024,
        attr: SECTION_ATTR_RAM,
    },
    SectionMap {
        start: 0xE000_0000,
        size_mib: 16,
        attr: SECTION_ATTR_DEVICE,
    },
    SectionMap {
        start: 0xF800_0000,
        size_mib: 16,
        attr: SECTION_ATTR_DEVICE,
    },
    //
    // Sections for PL
    //
    SectionMap {
        start: 0x4120_0000,
        size_mib: 1,
        attr: SECTION_ATTR_DEVICE,
    },
    SectionMap {
        start: 0x42C0_0000,
        size_mib: 1,
        attr: SECTION_ATTR_DEVICE,
    },
];

const fn map_section(descs: &mut [L1Section; NUM_L1_PAGE_TABLE_ENTRIES], map: &SectionMap) {
    let mut addr = (map.start >> 20) as u16;
    let addr_end = addr + map.size_mib;
    while addr < addr_end {
        let idx = addr as usize;
        descs[idx] = L1Section::new_with_addr_upper_bits_and_attrs(u12::new(addr), map.attr);
        addr += 1;
    }
}

const fn build_initial_l1_table() -> L1Table {
    let mut descs: [L1Section; NUM_L1_PAGE_TABLE_ENTRIES] =
        [L1Section::ZERO; NUM_L1_PAGE_TABLE_ENTRIES];

    let mut i = 0;
    while i < INITIAL_MAP.len() {
        map_section(&mut descs, &(INITIAL_MAP[i]));
        i += 1;
    }

    L1Table {
        entries: core::cell::UnsafeCell::new(descs),
    }
}

pub fn init() {
    let ttbr0 = aarch32_cpu::register::Ttbr0::new_with_raw_value(0)
        .with_address(core::ptr::addr_of!(SYS_L1_PAGE_TABLE) as usize)
        .with_irgn(false)
        .with_nos(false)
        .with_rgn(aarch32_cpu::register::ttbr0::Region::WriteBackWriteAllocCacheable)
        .with_s(true)
        .with_c(true);
    unsafe { aarch32_cpu::register::Ttbr0::write(ttbr0) }
}

const ACTLR_SMP_FW: u32 = (1 << 6) | 1;
const SCTLR_IZCM: u32 = (1 << 12) | (1 << 11) | (1 << 2) | 1;

pub fn enable_mmu_and_cache() {
    unsafe {
        core::arch::asm!(
            r#"
                mov     r0, #0xFFFFFFFF
                mcr     p15, 0, r0, c3, c0, 0   // Write DACR

                movw    r0, #{actlr}
                mcr     p15, 0, r0, c1, c0, 1   // Write ACTLR

                mrc     p15, 0, r0, c1, c0, 0   // Read SCTLR
                movw    r1, #{sctlr_izcm}
                orr     r0, r0, r1

                dsb
                isb
                mcr     p15, 0, r0, c1, c0, 0   // Read SCTLR
                isb

                // Invalidate all TLB
                mcr     p15, 0, r0, c8, c7, 0   // TLBIALL
                dsb
                isb

                // Invalidate L1 I-$ and branch predictor
                mcr     p15, 0, r0, c7, c5, 0   // ICIALLU
                dsb
                isb
            "#,
            actlr = const ACTLR_SMP_FW,
            sctlr_izcm = const SCTLR_IZCM,
        );
    }
}

pub fn debug() {
    let entries = unsafe { &*SYS_L1_PAGE_TABLE.entries.get() };

    for e in entries.iter().enumerate() {
        crate::println!("{:03X}: {:?}", e.0, e.1);
    }
}
