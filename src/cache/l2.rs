use core::cell::RefCell;
use critical_section::Mutex;

static L2_CACHE: Mutex<RefCell<Option<L2Cache>>> = Mutex::new(RefCell::new(None));

struct L2Cache {
    regs: zynq7000::l2_cache::MmioRegisters<'static>,
}

impl L2Cache {
    pub unsafe fn new() -> L2Cache {
        L2Cache {
            regs: unsafe { zynq7000::l2_cache::Registers::new_mmio_fixed() },
        }
    }
}

pub fn clean(addr: u32) {
    critical_section::with(|cs| {
        if let Some(l2) = L2_CACHE.borrow(cs).borrow_mut().as_mut() {
            l2.regs.write_clean_by_pa(addr);
            while l2.regs.read_cache_sync().busy() {
                core::hint::spin_loop();
            }
            core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        };
    });    
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
