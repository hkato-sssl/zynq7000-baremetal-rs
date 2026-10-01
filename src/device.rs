use crate::driver::xpsuart::XPSUart;
use core::cell::RefCell;
use critical_section::Mutex;

type Gic = zynq7000_hal::gic::Configurator;

pub static XPSUART1: Mutex<RefCell<Option<XPSUart>>> = Mutex::new(RefCell::new(None));
pub static GIC: Mutex<RefCell<Option<Gic>>> = Mutex::new(RefCell::new(None));

fn create_gic() -> Gic {
    let gicc = unsafe { zynq7000::gic::CpuInterfaceRegisters::new_mmio_fixed() };
    let gicd = unsafe { zynq7000::gic::DistributorRegisters::new_mmio_fixed() };

    Gic::new_with_init(gicc, gicd)
}

fn create_resources() {
    critical_section::with(|cs| unsafe {
        XPSUART1.borrow(cs).replace(Some(XPSUart::new_fixed_1()));
        GIC.borrow(cs).replace(Some(create_gic()));
    });
}

fn init_gic() {
    critical_section::with(|cs| {
        if let Some(gic) = GIC.borrow(cs).borrow_mut().as_mut() {
            gic.initialize();
        }
    });
}

pub fn init() {
    if aarch32_cpu::asm::core_id() == 0 {
        create_resources();
        init_gic();
    }
}
