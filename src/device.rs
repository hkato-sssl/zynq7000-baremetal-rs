use crate::driver::xpsuart::XPSUart;
use core::cell::RefCell;
use critical_section::Mutex;

pub static XPSUART1: Mutex<RefCell<Option<XPSUart>>> = Mutex::new(RefCell::new(None));

fn create_resources() {
    critical_section::with(|cs| unsafe {
        XPSUART1.borrow(cs).replace(Some(XPSUart::new_fixed_1()));
    });
}

pub fn init() {
    if aarch32_cpu::asm::core_id() == 0 {
        create_resources();
    }
}
