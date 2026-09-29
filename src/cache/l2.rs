use core::cell::RefCell;
use core::sync::atomic::Ordering;
use core::sync::atomic::compiler_fence;
use critical_section::Mutex;
use zynq7000::l2_cache::{MmioRegisters, Registers};

static L2_CACHE: Mutex<RefCell<Option<L2Cache>>> = Mutex::new(RefCell::new(None));

const L2_CACHE_LINE_SIZE: u32 = 32;

struct L2Cache {
    regs: MmioRegisters<'static>,
}

impl L2Cache {
    pub unsafe fn new() -> L2Cache {
        L2Cache {
            regs: unsafe { Registers::new_mmio_fixed() },
        }
    }
}

pub fn init() {
    critical_section::with(|cs| unsafe {
        if L2_CACHE.borrow(cs).borrow().is_none() {
            L2_CACHE.borrow(cs).replace(Some(L2Cache::new()));
        }

        if let Some(l2) = L2_CACHE.borrow(cs).borrow_mut().as_mut() {
            zynq7000_hal::l2_cache::init(
                &mut l2.regs,
                zynq7000_hal::l2_cache::DEFAULT_TAG_RAM_LATENCY,
                zynq7000_hal::l2_cache::DEFAULT_DATA_RAM_LATENCY,
            );
        }
    });
}

pub fn clean_all() {
    maintenance_all(MmioRegisters::write_clean_by_way);
}

pub fn invalidate_all() {
    maintenance_all(MmioRegisters::write_invalidate_by_way);
}

pub fn clean_and_invalidate_all() {
    maintenance_all(MmioRegisters::write_clean_invalidate_by_way);
}

pub fn clean(addr: u32) {
    maintenance_pa(MmioRegisters::write_clean_by_pa, addr);
}

pub fn invalidate(addr: u32) {
    maintenance_pa(MmioRegisters::write_invalidate_by_pa, addr);
}

pub fn clean_and_invalidate(addr: u32) {
    maintenance_pa(MmioRegisters::write_clean_invalidate_by_pa, addr);
}

pub fn clean_range(addr: u32, size: u32) {
    if is_valid_range(addr, size) {
        maintenance_range_pa(MmioRegisters::write_clean_by_pa, addr, size);
    }
}

pub fn invalidate_range(addr: u32, size: u32) {
    if is_valid_range(addr, size) {
        maintenance_range_pa(MmioRegisters::write_invalidate_by_pa, addr, size);
    }
}

pub fn clean_and_invalidate_range(addr: u32, size: u32) {
    if is_valid_range(addr, size) {
        maintenance_range_pa(MmioRegisters::write_clean_invalidate_by_pa, addr, size);
    }
}

fn maintenance_all<F>(ops: F)
where
    F: FnOnce(&mut MmioRegisters<'static>, u32),
{
    const ALL_WAYS: u32 = 0xffff;

    compiler_fence(Ordering::SeqCst);
    critical_section::with(|cs| {
        if let Some(l2) = L2_CACHE.borrow(cs).borrow_mut().as_mut() {
            ops(&mut l2.regs, ALL_WAYS);
            while l2.regs.read_cache_sync().busy() {
                core::hint::spin_loop();
            }
        }
    });
    compiler_fence(Ordering::SeqCst);
}

fn maintenance_pa<F>(ops: F, addr: u32)
where
    F: FnOnce(&mut MmioRegisters<'static>, u32),
{
    compiler_fence(Ordering::SeqCst);
    critical_section::with(|cs| {
        if let Some(l2) = L2_CACHE.borrow(cs).borrow_mut().as_mut() {
            ops(&mut l2.regs, addr);
            while l2.regs.read_cache_sync().busy() {
                core::hint::spin_loop();
            }
        }
    });
    compiler_fence(Ordering::SeqCst);
}

fn aligned_addr(addr: u32) -> u32 {
    addr & !(L2_CACHE_LINE_SIZE - 1)
}

fn is_valid_range(addr: u32, size: u32) -> bool {
    (size > 0) && !(addr.checked_add(size - 1).is_none())
}

fn maintenance_range_pa<F>(ops: F, addr: u32, size: u32)
where
    F: Fn(&mut MmioRegisters<'static>, u32),
{
    let st = aligned_addr(addr);
    let ed = aligned_addr(addr + (size - 1));

    compiler_fence(Ordering::SeqCst);
    critical_section::with(|cs| {
        if let Some(l2) = L2_CACHE.borrow(cs).borrow_mut().as_mut() {
            let mut addr = st;
            loop {
                ops(&mut l2.regs, addr);
                if addr == ed {
                    break;
                }
                addr += L2_CACHE_LINE_SIZE;
            }
            while l2.regs.read_cache_sync().busy() {
                core::hint::spin_loop();
            }
        }
    });
    compiler_fence(Ordering::SeqCst);
}
