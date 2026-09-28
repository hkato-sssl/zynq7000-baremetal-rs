use crate::driver::uartlite::UartLite;

pub use device::*;
pub use register::*;

#[allow(unused)]
pub(crate) mod register {
    pub const PL_GPIO0_BASE_ADDR: usize = 0x4120_0000;
    pub const PL_GPIO1_BASE_ADDR: usize = 0x4121_0000;
    pub const PL_UARTLITE0_BASE_ADDR: usize = 0x42C0_0000;
    pub const PL_UARTLITE1_BASE_ADDR: usize = 0x42C1_0000;
}

pub mod device {
    use crate::driver::uartlite::UartLite;
    use core::cell::RefCell;
    use critical_section::Mutex;

    pub static UARTLITE0: Mutex<RefCell<Option<UartLite>>> = Mutex::new(RefCell::new(None));
    pub static UARTLITE1: Mutex<RefCell<Option<UartLite>>> = Mutex::new(RefCell::new(None));
}

fn create_resources() {
    critical_section::with(|cs| unsafe {
        UARTLITE0
            .borrow(cs)
            .replace(Some(UartLite::new(PL_UARTLITE0_BASE_ADDR)));
        UARTLITE1
            .borrow(cs)
            .replace(Some(UartLite::new(PL_UARTLITE1_BASE_ADDR)));
    });
}

pub fn init() {
    if aarch32_cpu::asm::core_id() == 0 {
        create_resources();
    }
}
