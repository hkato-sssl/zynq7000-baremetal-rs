pub mod l1;
pub mod l2;

pub fn clean(addr: u32) {
    l1::d_clean(addr);
    aarch32_cpu::asm::dsb();
    l2::clean(addr);
}