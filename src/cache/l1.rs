pub use aarch32_cpu::cache::clean_and_invalidate_data_cache_line_to_poc as d_clean_and_invalidate;
pub use aarch32_cpu::cache::clean_data_cache_line_to_poc as d_clean;
pub use aarch32_cpu::cache::invalidate_data_cache_line_to_poc as d_invalidate;

// Configuration parameters for ZYNQ-7000 PS
const L1_WAY: usize = 2; // log2(# of ways)
const L1_SETS: usize = 8; // log2(# of sets)
const L1_LINE_SIZE: usize = 5; // log2(line size)

const L1_CACHE_LINE_SIZE: u32 = 1u32 << L1_LINE_SIZE;

pub fn d_invalidate_all() {
    aarch32_cpu::cache::invalidate_l1_data_cache::<L1_WAY, L1_LINE_SIZE, L1_SETS>();
}

pub fn d_clean_all() {
    aarch32_cpu::cache::clean_l1_data_cache::<L1_WAY, L1_LINE_SIZE, L1_SETS>();
}

pub fn d_clean_and_invalidate_all() {
    aarch32_cpu::cache::clean_and_invalidate_l1_data_cache::<L1_WAY, L1_LINE_SIZE, L1_SETS>();
}

pub fn d_clean_range(addr: u32, size: u32) {
    if is_valid_range(addr, size) {
        maintenance_range(d_clean, addr, size);
    }
}

pub fn d_invalidate_range(addr: u32, size: u32) {
    if is_valid_range(addr, size) {
        maintenance_range(d_invalidate, addr, size);
    }
}

pub fn d_clean_and_invalidate_range(addr: u32, size: u32) {
    if is_valid_range(addr, size) {
        maintenance_range(d_clean_and_invalidate, addr, size);
    }
}

fn aligned_addr(addr: u32) -> u32 {
    addr & !(L1_CACHE_LINE_SIZE - 1)
}

fn is_valid_range(addr: u32, size: u32) -> bool {
    (size > 0) && !(addr.checked_add(size - 1).is_none())
}

fn maintenance_range<F>(ops: F, addr: u32, size: u32)
where
    F: Fn(u32),
{
    let st = aligned_addr(addr);
    let ed = aligned_addr(addr + (size - 1));
    let mut addr = st;

    loop {
        ops(addr);
        if addr == ed {
            break;
        }
        addr += L1_CACHE_LINE_SIZE;
    }
}
