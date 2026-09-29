pub mod l1;
pub mod l2;

pub fn d_clean_all() {
    l1::d_clean_all();
    aarch32_cpu::asm::dsb();
    l2::clean_all();
}

pub fn d_invalidate_all() {
    l1::d_invalidate_all();
    aarch32_cpu::asm::dsb();
    l2::invalidate_all();
}

pub fn d_clean_and_invalidate_all() {
    l1::d_clean_and_invalidate_all();
    aarch32_cpu::asm::dsb();
    l2::clean_and_invalidate_all();
}

pub fn d_clean(addr: u32) {
    l1::d_clean(addr);
    aarch32_cpu::asm::dsb();
    l2::clean(addr);
}

pub fn d_invalidate(addr: u32) {
    l1::d_invalidate(addr);
    aarch32_cpu::asm::dsb();
    l2::invalidate(addr);
}

pub fn d_clean_and_invalidate(addr: u32) {
    l1::d_clean_and_invalidate(addr);
    aarch32_cpu::asm::dsb();
    l2::clean_and_invalidate(addr);
}

pub fn d_clean_range(addr: u32, size: u32) {
    l1::d_clean_range(addr, size);
    aarch32_cpu::asm::dsb();
    l2::clean_range(addr, size);
}

pub fn d_invalidate_range(addr: u32, size: u32) {
    l1::d_invalidate_range(addr, size);
    aarch32_cpu::asm::dsb();
    l2::invalidate_range(addr, size);
}

pub fn d_clean_and_invalidate_range(addr: u32, size: u32) {
    l1::d_clean_and_invalidate_range(addr, size);
    aarch32_cpu::asm::dsb();
    l2::clean_and_invalidate_range(addr, size);
}
