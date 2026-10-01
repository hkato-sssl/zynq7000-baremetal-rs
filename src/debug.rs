pub mod rgb {
    use crate::driver::axi_gpio::AxiGpio;
    use crate::pl::register::PL_GPIO0_BASE_ADDR;

    fn rgb(r: bool, g: bool, b: bool) -> u32 {
        let mut d: u32;
        d = u32::from(r) << 2;
        d |= u32::from(g) << 1;
        d |= u32::from(b);
        d
    }

    pub unsafe fn set(no: u32, r: bool, g: bool, b: bool) {
        let mut gpio = unsafe { AxiGpio::new(PL_GPIO0_BASE_ADDR) };
        let d = rgb(r, g, b);
        match no {
            0 => gpio.set2(d),
            1 => gpio.set2(d << 3),
            _ => (),
        }
    }

    pub unsafe fn clear(no: u32, r: bool, g: bool, b: bool) {
        let mut gpio = unsafe { AxiGpio::new(PL_GPIO0_BASE_ADDR) };
        let d = rgb(r, g, b);
        match no {
            0 => gpio.clear2(d),
            1 => gpio.clear2(d << 3),
            _ => (),
        }
    }
}
