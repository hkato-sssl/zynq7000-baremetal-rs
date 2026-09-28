// Configuration parameters for ZYNQ-7000 PS
const L1_WAY: usize = 2; // log2(# of ways)
const L1_SETS: usize = 8; // log2(# of sets)
const L1_LINE_SIZE: usize = 5; // log2(line size)

pub use aarch32_cpu::cache::clean_and_invalidate_data_cache_line_to_poc as d_clean_and_invalidate;
pub use aarch32_cpu::cache::clean_data_cache_line_to_poc as d_clean;
pub use aarch32_cpu::cache::invalidate_data_cache_line_to_poc as d_invalidate;

pub fn d_invalidate_all() {
    aarch32_cpu::cache::invalidate_l1_data_cache::<L1_WAY, L1_LINE_SIZE, L1_SETS>();
}

pub fn d_clean_all() {
    aarch32_cpu::cache::clean_l1_data_cache::<L1_WAY, L1_LINE_SIZE, L1_SETS>();
}

pub fn d_clean_and_invalidate_all() {
    aarch32_cpu::cache::clean_and_invalidate_l1_data_cache::<L1_WAY, L1_LINE_SIZE, L1_SETS>();
}
