use crate::cpu::register::TlbIMvaIs;
use crate::mmu::pool::sys_alloc_page_table;
use arbitrary_int::{u2, u3, u4, u12, u20, u22};
use core::cell::RefCell;
use core::cell::UnsafeCell;
use core::result::Result;
use critical_section::Mutex;

use crate::mmu::*;

pub static SYS_L1_PAGE_TABLE: L1Table<MAX_L1_ENTRIES> = L1Table {
    entries: UnsafeCell::new([0u32; MAX_L1_ENTRIES]),
};

pub static SYS_MMU: Mutex<RefCell<Option<&L1Table<MAX_L1_ENTRIES>>>> =
    Mutex::new(RefCell::new(None));

#[derive(Debug)]
pub enum MmuError {
    UnalignedAddress,
    UnalignedSize,
    OutOfRange,
    InvalidRange,
    NoFreePages,
    AlreadyMapped,
    SecureFault,
    Others,
}

#[derive(PartialEq, Eq)]
enum L1Descriptor {
    Invalid,
    Section,
    PageTable(PageTableDesc),
    SuperSection,
    Reserved,
}

impl L1Descriptor {
    fn new(desc: u32) -> Self {
        match desc & 0b11 {
            0b00 => L1Descriptor::Invalid,
            0b01 => L1Descriptor::PageTable(PageTableDesc::new_with_raw_value(desc)),
            0b10 => {
                if (desc & 0x40000) == 0 {
                    L1Descriptor::Section
                } else {
                    L1Descriptor::SuperSection
                }
            }
            _ => L1Descriptor::Reserved,
        }
    }
}

#[derive(PartialEq, Eq)]
enum L2Descriptor {
    Invalid,
    LargePage,
    Page,
}

impl L2Descriptor {
    fn new(desc: u32) -> Self {
        match desc & 0b11 {
            0b00 => L2Descriptor::Invalid,
            0b01 => L2Descriptor::LargePage,
            _ => L2Descriptor::Page,
        }
    }
}

struct MemAttrs {
    tex: u3,
    b: bool,
    c: bool,
}

impl MemAttrs {
    const fn decode(attr: &MemoryRegionAttributes) -> Self {
        match attr {
            StronglyOrdered => MemAttrs {
                tex: u3::new(0b000),
                c: false,
                b: false,
            },
            ShareableDevice => MemAttrs {
                tex: u3::new(0b000),
                c: false,
                b: true,
            },
            OuterAndInnerWriteThroughNoWriteAlloc => MemAttrs {
                tex: u3::new(0b000),
                c: true,
                b: false,
            },
            OuterAndInnerWriteBackNoWriteAlloc => MemAttrs {
                tex: u3::new(0b000),
                c: true,
                b: true,
            },
            OuterAndInnerNonCacheable => MemAttrs {
                tex: u3::new(0b001),
                c: false,
                b: false,
            },
            OuterAndInnerWriteBackWriteAlloc => MemAttrs {
                tex: u3::new(0b001),
                c: true,
                b: true,
            },
            NonShareableDevice => MemAttrs {
                tex: u3::new(0b010),
                c: false,
                b: false,
            },
            CacheableMemory { inner, outer } => MemAttrs {
                tex: u3::new(0b100 | *outer as u8),
                c: (*inner as u8 & 0b10) != 0,
                b: (*inner as u8 & 0b01) != 0,
            },
        }
    }
}

#[derive(Debug)]
pub struct MapAttributes {
    /// Non-secure bit
    pub non_secure: bool,
    /// nG bit
    pub non_global: bool,
    /// Shareable bit
    pub shareable: bool,
    /// Access permissions bits
    pub access: AccessPermissions,
    /// Memory region attribute
    pub memory_attrs: MemoryRegionAttributes,
    /// Domain field
    pub domain: u4,
    /// Execute-never bit
    pub execute_never: bool,
}

#[bitbybit::bitfield(u32, default = 0b10, defmt_fields(feature = "defmt"))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq)]
pub struct SectionDesc {
    /// SectionDesc base address upper bits.
    #[bits(20..=31, rw)]
    base_addr: u12,
    /// Non-secure bit.
    #[bit(19, rw)]
    ns: bool,
    /// Non-global bit.
    #[bit(17, rw)]
    ng: bool,
    /// Shareable bit.
    #[bit(16, rw)]
    s: bool,
    /// Access permissions bit[2].
    #[bit(15, rw)]
    apx: bool,
    /// Part of memory region attributes.
    #[bits(12..=14, rw)]
    tex: u3,
    /// Access permissions bits[1:0].
    #[bits(10..=11, rw)]
    ap: u2,
    /// Domain field.
    #[bits(5..=8, rw)]
    domain: u4,
    /// Execute-never bit.
    #[bit(4, rw)]
    xn: bool,
    /// Part of memory region attributes.
    #[bit(3, rw)]
    c: bool,
    /// Part of memory region attributes.
    #[bit(2, rw)]
    b: bool,
}

impl SectionDesc {
    pub fn new_with_addr(paddr: usize, attrs: &MapAttributes) -> Self {
        let mem_attrs = MemAttrs::decode(&attrs.memory_attrs);
        // AP=0b111 is not supported on ZYNQ-7000
        let ap = if attrs.access == AccessPermissions::ReadOnly {
            AccessPermissions::ReadOnlyv6
        } else {
            attrs.access
        };
        Self::builder()
            .with_base_addr(u12::new((paddr >> 20) as u16))
            .with_ns(attrs.non_secure)
            .with_ng(attrs.non_global)
            .with_s(attrs.shareable)
            .with_apx(ap.apx())
            .with_tex(mem_attrs.tex)
            .with_ap(ap.ap())
            .with_domain(attrs.domain)
            .with_xn(attrs.execute_never)
            .with_c(mem_attrs.c)
            .with_b(mem_attrs.b)
            .build()
    }
}

#[bitbybit::bitfield(u32, default = 0x40002, defmt_fields(feature = "defmt"))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq)]
pub struct SuperSectionDesc {
    /// SectionDesc base address upper bits.
    #[bits(24..=31, rw)]
    base_addr: u8,
    /// Non-secure bit.
    #[bit(19, rw)]
    ns: bool,
    /// Non-global bit.
    #[bit(17, rw)]
    ng: bool,
    /// Shareable bit.
    #[bit(16, rw)]
    s: bool,
    /// Access permissions bit[2].
    #[bit(15, rw)]
    apx: bool,
    /// Part of memory region attributes.
    #[bits(12..=14, rw)]
    tex: u3,
    /// Access permissions bits[1:0].
    #[bits(10..=11, rw)]
    ap: u2,
    /// Execute-never bit.
    #[bit(4, rw)]
    xn: bool,
    /// Part of memory region attributes.
    #[bit(3, rw)]
    c: bool,
    /// Part of memory region attributes.
    #[bit(2, rw)]
    b: bool,
}

impl SuperSectionDesc {
    pub fn new_with_addr(paddr: usize, attrs: &MapAttributes) -> Self {
        #[cfg(debug_assertions)]
        if attrs.domain != u4::new(0) {
            panic!("Unsupported domain");
        }
        let mem_attrs = MemAttrs::decode(&attrs.memory_attrs);
        // AP=0b111 is not supported on ZYNQ-7000
        let ap = if attrs.access == AccessPermissions::ReadOnly {
            AccessPermissions::ReadOnlyv6
        } else {
            attrs.access
        };
        Self::builder()
            .with_base_addr((paddr >> 24) as u8)
            .with_ns(attrs.non_secure)
            .with_ng(attrs.non_global)
            .with_s(attrs.shareable)
            .with_apx(ap.apx())
            .with_tex(mem_attrs.tex)
            .with_ap(ap.ap())
            .with_xn(attrs.execute_never)
            .with_c(mem_attrs.c)
            .with_b(mem_attrs.b)
            .build()
    }
}

#[bitbybit::bitfield(u32, default = 0b01, defmt_fields(feature = "defmt"))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq)]
pub struct PageTableDesc {
    /// Page table base address.
    #[bits(10..=31, rw)]
    base_addr: u22,
    /// Domain field.
    #[bits(5..=8, rw)]
    domain: u4,
    /// Non-secure bit.
    #[bit(3, rw)]
    ns: bool,
}

impl PageTableDesc {
    fn new_with_page_table(page_table: &PageTable, attrs: &MapAttributes) -> Self {
        let addr = page_table.descs.as_ptr() as usize;
        Self::builder()
            .with_base_addr(u22::new((addr >> 10) as u32))
            .with_domain(attrs.domain)
            .with_ns(attrs.non_secure)
            .build()
    }
}

#[bitbybit::bitfield(u32, default = 0b10, defmt_fields(feature = "defmt"))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq)]
pub struct PageDesc {
    /// Page base address.
    #[bits(12..=31, rw)]
    base_addr: u20,
    /// Non-global bit.
    #[bit(11, rw)]
    ng: bool,
    /// Shareable bit.
    #[bit(10, rw)]
    s: bool,
    /// Access permissions bit[2].
    #[bit(9, rw)]
    apx: bool,
    /// Part of memory region attributes.
    #[bits(6..=8, rw)]
    tex: u3,
    /// Access permissions bits[1:0].
    #[bits(4..=5, rw)]
    ap: u2,
    /// Part of memory region attributes.
    #[bit(3, rw)]
    c: bool,
    /// Part of memory region attributes.
    #[bit(2, rw)]
    b: bool,
    /// Execute-never bit.
    #[bit(0, rw)]
    xn: bool,
}

impl PageDesc {
    pub fn new_with_addr(addr: usize, attrs: &MapAttributes) -> Self {
        let mem_attrs = MemAttrs::decode(&attrs.memory_attrs);
        // AP=0b111 is not supported on ZYNQ-7000
        let ap = if attrs.access == AccessPermissions::ReadOnly {
            AccessPermissions::ReadOnlyv6
        } else {
            attrs.access
        };
        Self::builder()
            .with_base_addr(u20::new((addr >> 12) as u32))
            .with_ng(attrs.non_global)
            .with_s(attrs.shareable)
            .with_apx(ap.apx())
            .with_tex(mem_attrs.tex)
            .with_ap(ap.ap())
            .with_c(mem_attrs.c)
            .with_b(mem_attrs.b)
            .with_xn(attrs.execute_never)
            .build()
    }
}

#[bitbybit::bitfield(u32, default = 0b01, defmt_fields(feature = "defmt"))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq)]
pub struct LargePageDesc {
    /// Page base address.
    #[bits(16..=31, rw)]
    base_addr: u16,
    /// Execute-never bit.
    #[bit(15, rw)]
    xn: bool,
    /// Part of memory region attributes.
    #[bits(12..=14, rw)]
    tex: u3,
    /// Non-global bit.
    #[bit(11, rw)]
    ng: bool,
    /// Shareable bit.
    #[bit(10, rw)]
    s: bool,
    /// Access permissions bit[2].
    #[bit(9, rw)]
    apx: bool,
    /// Access permissions bits[1:0].
    #[bits(4..=5, rw)]
    ap: u2,
    /// Part of memory region attributes.
    #[bit(3, rw)]
    c: bool,
    /// Part of memory region attributes.
    #[bit(2, rw)]
    b: bool,
}

impl LargePageDesc {
    pub fn new_with_addr(addr: usize, attrs: &MapAttributes) -> Self {
        let mem_attrs = MemAttrs::decode(&attrs.memory_attrs);
        // AP=0b111 is not supported on ZYNQ-7000
        let ap = if attrs.access == AccessPermissions::ReadOnly {
            AccessPermissions::ReadOnlyv6
        } else {
            attrs.access
        };
        Self::builder()
            .with_base_addr((addr >> 16) as u16)
            .with_xn(attrs.execute_never)
            .with_tex(mem_attrs.tex)
            .with_ng(attrs.non_global)
            .with_s(attrs.shareable)
            .with_apx(ap.apx())
            .with_ap(ap.ap())
            .with_c(mem_attrs.c)
            .with_b(mem_attrs.b)
            .build()
    }
}

pub struct PageTable<'a> {
    pub descs: &'a mut [u32; NUM_L2_ENTRIES],
}

impl<'a> PageTable<'a> {
    fn new_with_descriptor(desc: PageTableDesc) -> Self {
        let addr = ((desc.base_addr().value() as usize) << 10) as *mut [u32; NUM_L2_ENTRIES];
        Self::new_with_addr(addr)
    }

    fn new_with_addr(addr: *mut [u32; NUM_L2_ENTRIES]) -> Self {
        PageTable {
            descs: unsafe { &mut *addr },
        }
    }

    fn map_large_page(
        &mut self,
        vaddr: usize,
        paddr: usize,
        attrs: &MapAttributes,
    ) -> Result<(), MmuError> {
        let idx = l2_index(vaddr);
        for i in idx..(idx + 16) {
            let desc = L2Descriptor::new(self.descs[i]);
            if desc != L2Descriptor::Invalid {
                return Err(MmuError::AlreadyMapped);
            }
        }

        let desc = LargePageDesc::new_with_addr(paddr, attrs).raw_value();
        for i in idx..(idx + 16) {
            self.write_desc(i, desc);
        }
        Ok(())
    }

    fn map_page(
        &mut self,
        vaddr: usize,
        paddr: usize,
        attrs: &MapAttributes,
    ) -> Result<(), MmuError> {
        let idx = l2_index(vaddr);
        let desc = L2Descriptor::new(self.descs[idx]);
        match desc {
            L2Descriptor::Invalid => {
                let desc = PageDesc::new_with_addr(paddr, attrs);
                self.write_desc(idx, desc.raw_value());
                Ok(())
            }
            _ => Err(MmuError::AlreadyMapped),
        }
    }

    fn write_desc(&mut self, idx: usize, desc: u32) {
        let descs = self.descs.as_mut_ptr() as *mut u32;
        let addr = unsafe { descs.add(idx) };
        unsafe { addr.write_volatile(desc) };

        if is_mmu_enabled() {
            crate::cache::d_clean(addr as u32);
        }
    }
}

#[repr(C, align(0x4000))] // 16KiB alignment
#[derive(Debug)]
pub struct L1Table<const N: usize> {
    pub entries: UnsafeCell<[u32; N]>,
}

impl<const N: usize> L1Table<N> {
    fn entries(&self) -> &mut [u32; N] {
        unsafe { &mut *self.entries.get() }
    }

    pub fn map(
        &self,
        vaddr: usize,
        paddr: usize,
        size: usize,
        attrs: &MapAttributes,
        remap: bool,
    ) -> Result<(), MmuError> {
        if !is_aligned(vaddr, PAGE_SIZE) || !is_aligned(paddr, PAGE_SIZE) {
            Err(MmuError::UnalignedAddress)
        } else if !is_aligned(size, PAGE_SIZE) {
            Err(MmuError::UnalignedSize)
        } else if !Self::in_region(vaddr, size) {
            Err(MmuError::OutOfRange)
        } else if !is_valid_size(vaddr, size) {
            Err(MmuError::InvalidRange)
        } else if !is_valid_size(paddr, size) {
            Err(MmuError::InvalidRange)
        } else {
            self.map_regions(vaddr, paddr, size, attrs, remap)
        }
    }

    fn map_regions(
        &self,
        mut vaddr: usize,
        mut paddr: usize,
        mut size: usize,
        attrs: &MapAttributes,
        remap: bool,
    ) -> Result<(), MmuError> {
        while size > 0 {
            if !remap
                && (size >= SUPER_SECTION_SIZE)
                && is_aligned(vaddr, SUPER_SECTION_SIZE)
                && is_aligned(paddr, SUPER_SECTION_SIZE)
            {
                self.map_super_section(vaddr, paddr, attrs)?;
                vaddr += SUPER_SECTION_SIZE;
                paddr += SUPER_SECTION_SIZE;
                size -= SUPER_SECTION_SIZE;
            } else if (size >= SECTION_SIZE)
                && is_aligned(vaddr, SECTION_SIZE)
                && is_aligned(paddr, SECTION_SIZE)
            {
                self.map_section(vaddr, paddr, attrs, remap)?;
                vaddr += SECTION_SIZE;
                paddr += SECTION_SIZE;
                size -= SECTION_SIZE;
            } else if !remap
                && (size >= LARGE_PAGE_SIZE)
                && is_aligned(vaddr, LARGE_PAGE_SIZE)
                && is_aligned(paddr, LARGE_PAGE_SIZE)
            {
                self.map_large_page(vaddr, paddr, attrs)?;
                vaddr += LARGE_PAGE_SIZE;
                paddr += LARGE_PAGE_SIZE;
                size -= LARGE_PAGE_SIZE;
            } else {
                self.map_page(vaddr, paddr, attrs)?;
                vaddr += PAGE_SIZE;
                paddr += PAGE_SIZE;
                size -= PAGE_SIZE;
            }
        }
        Ok(())
    }

    fn map_super_section(
        &self,
        vaddr: usize,
        paddr: usize,
        attrs: &MapAttributes,
    ) -> Result<(), MmuError> {
        let entries = self.entries();
        let idx = l1_index(vaddr);
        for i in idx..(idx + 16) {
            let desc = L1Descriptor::new(entries[i]);
            if desc != L1Descriptor::Invalid {
                return Err(MmuError::AlreadyMapped);
            }
        }

        let desc = SuperSectionDesc::new_with_addr(paddr, attrs).raw_value();
        for i in idx..(idx + 16) {
            self.write_desc(i, desc);
        }
        Ok(())
    }

    fn map_section(
        &self,
        vaddr: usize,
        paddr: usize,
        attrs: &MapAttributes,
        remap: bool,
    ) -> Result<(), MmuError> {
        let entries = self.entries();
        let idx = l1_index(vaddr);
        let desc = L1Descriptor::new(entries[idx]);
        match desc {
            L1Descriptor::Invalid => {
                let section = SectionDesc::new_with_addr(paddr, attrs);
                self.write_desc(idx, section.raw_value());
                Ok(())
            }
            L1Descriptor::PageTable(_) => Err(MmuError::AlreadyMapped),
            L1Descriptor::Section => {
                if remap {
                    self.update_section(vaddr, paddr, attrs);
                    Ok(())
                } else {
                    Err(MmuError::AlreadyMapped)
                }
            }
            L1Descriptor::SuperSection => Err(MmuError::AlreadyMapped),
            _ => Err(MmuError::Others),
        }
    }

    fn map_large_page(
        &self,
        vaddr: usize,
        paddr: usize,
        attrs: &MapAttributes,
    ) -> Result<(), MmuError> {
        let entries = self.entries();
        let idx = l1_index(vaddr);
        let desc = L1Descriptor::new(entries[idx]);
        match desc {
            L1Descriptor::PageTable(desc) => {
                if desc.ns() == attrs.non_secure && desc.domain() == attrs.domain {
                    let mut page_table = PageTable::new_with_descriptor(desc);
                    page_table.map_large_page(vaddr, paddr, attrs)
                } else {
                    Err(MmuError::SecureFault)
                }
            }
            L1Descriptor::Invalid => {
                let mut page_table = sys_alloc_page_table()?;
                page_table.map_large_page(vaddr, paddr, attrs).map(|_| {
                    let desc = PageTableDesc::new_with_page_table(&page_table, attrs).raw_value();
                    self.write_desc(idx, desc)
                })
            }
            L1Descriptor::Section => Err(MmuError::AlreadyMapped),
            L1Descriptor::SuperSection => Err(MmuError::AlreadyMapped),
            _ => Err(MmuError::Others),
        }
    }

    fn map_page(&self, vaddr: usize, paddr: usize, attrs: &MapAttributes) -> Result<(), MmuError> {
        let entries = self.entries();
        let idx = l1_index(vaddr);
        let desc = L1Descriptor::new(entries[idx]);
        match desc {
            L1Descriptor::PageTable(desc) => {
                if desc.ns() == attrs.non_secure && desc.domain() == attrs.domain {
                    let mut page_table = PageTable::new_with_descriptor(desc);
                    page_table.map_page(vaddr, paddr, attrs)
                } else {
                    Err(MmuError::SecureFault)
                }
            }
            L1Descriptor::Invalid => {
                let mut page_table = sys_alloc_page_table()?;
                page_table.map_page(vaddr, paddr, attrs).map(|_| {
                    let desc = PageTableDesc::new_with_page_table(&page_table, attrs).raw_value();
                    self.write_desc(idx, desc)
                })
            }
            L1Descriptor::Section => Err(MmuError::AlreadyMapped),
            L1Descriptor::SuperSection => Err(MmuError::AlreadyMapped),
            _ => Err(MmuError::Others),
        }
    }

    fn _get_page_table(&self, vaddr: usize) -> Result<PageTable<'_>, MmuError> {
        let entries = self.entries();
        let idx = l1_index(vaddr);
        let desc = entries[idx];
        if (desc & 0b11) == 0b01 {
            let desc = PageTableDesc::new_with_raw_value(desc);
            Ok(PageTable::new_with_descriptor(desc))
        } else if (desc & 0b11) == 0b00 {
            sys_alloc_page_table()
        } else {
            Err(MmuError::AlreadyMapped)
        }
    }

    fn update_section(&self, vaddr: usize, paddr: usize, attrs: &MapAttributes) {
        const INVALID_SECTION: u32 = 0;
        let idx = l1_index(vaddr);
        let desc = SectionDesc::new_with_addr(paddr, attrs);

        // Update a section descriptor using break-before-make.

        self.write_desc(idx, INVALID_SECTION);
        TlbIMvaIs::write(vaddr as u32);
        aarch32_cpu::asm::dsb();
        self.write_desc(idx, desc.raw_value());
    }

    fn write_desc(&self, idx: usize, desc: u32) {
        let descs = self.entries.get() as *mut u32;
        let addr = unsafe { descs.add(idx) };
        unsafe { addr.write_volatile(desc) };

        if is_mmu_enabled() {
            crate::cache::d_clean(addr as u32);
        }
    }

    const fn in_region(addr: usize, size: usize) -> bool {
        let start_idx = l1_index(addr);
        let end_idx = l1_index(addr.wrapping_add(size).wrapping_sub(1));
        (start_idx < N) && (end_idx < N)
    }
}

unsafe impl<const N: usize> Sync for L1Table<N> {}

const fn is_aligned(value: usize, alignment: usize) -> bool {
    (alignment != 0) && (value % alignment == 0)
}

const fn is_valid_size(addr: usize, size: usize) -> bool {
    if size == 0 {
        false
    } else {
        let tail = addr.wrapping_add(size).wrapping_sub(1);
        addr < tail
    }
}

const fn l1_index(vaddr: usize) -> usize {
    vaddr / SECTION_SIZE
}

const fn l2_index(vaddr: usize) -> usize {
    (vaddr >> 12) & 0xFF
}

fn is_mmu_enabled() -> bool {
    let sctlr = aarch32_cpu::register::Sctlr::read();
    sctlr.c() && sctlr.m()
}

pub fn map(vaddr: usize, paddr: usize, size: usize, attrs: &MapAttributes) -> Result<(), MmuError> {
    critical_section::with(|cs| {
        if let Some(sys_mmu) = SYS_MMU.borrow(cs).borrow_mut().as_mut() {
            sys_mmu.map(vaddr, paddr, size, attrs, false)
        } else {
            Err(MmuError::Others)
        }
    })
}

pub fn remap(
    vaddr: usize,
    paddr: usize,
    size: usize,
    attrs: &MapAttributes,
) -> Result<(), MmuError> {
    critical_section::with(|cs| {
        if let Some(sys_mmu) = SYS_MMU.borrow(cs).borrow_mut().as_mut() {
            sys_mmu.map(vaddr, paddr, size, attrs, true)
        } else {
            Err(MmuError::Others)
        }
    })
}
