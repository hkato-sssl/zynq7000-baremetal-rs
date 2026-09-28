use crate::driver::axi_gpio::AxiGpio;
use crate::pl::register::PL_GPIO0_BASE_ADDR;
use core::cell::RefCell;

static mut DBG_GPIO: RefCell<Option<AxiGpio>> = RefCell::new(None);

fn rgb(r: bool, g: bool, b: bool) -> u32 {
    let mut d: u32;
    d = u32::from(r) << 2;
    d |= u32::from(g) << 1;
    d |= u32::from(b);
    d
}

#[allow(static_mut_refs)]
pub unsafe fn set_rgb(no: u32, r: bool, g: bool, b: bool) {
    unsafe {
        if let Some(gpio) = DBG_GPIO.get_mut() {
            let d = rgb(r, g, b);
            match no {
                0 => gpio.set2(d),
                1 => gpio.set2(d << 3),
                _ => (),
            }
        }
    }
}

#[allow(static_mut_refs)]
pub unsafe fn clear_rgb(no: u32, r: bool, g: bool, b: bool) {
    unsafe {
        if let Some(gpio) = DBG_GPIO.get_mut() {
            let d = rgb(r, g, b);
            match no {
                0 => gpio.clear2(d),
                1 => gpio.clear2(d << 3),
                _ => (),
            }
        }
    }
}

#[allow(static_mut_refs)]
fn create_resources() {
    unsafe {
        if DBG_GPIO.borrow().is_none() {
            DBG_GPIO.replace(Some(AxiGpio::new(PL_GPIO0_BASE_ADDR)));
        }
    }
}

pub fn init() {
    if aarch32_cpu::asm::core_id() == 0 {
        create_resources();
    }
}
