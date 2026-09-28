const CR: char = '\r';
const LF: char = '\n';

pub struct XPSUart {
    regs: zynq7000::uart::MmioRegisters<'static>,
}

impl XPSUart {
    pub unsafe fn new_fixed_0() -> XPSUart {
        XPSUart {
            regs: unsafe { zynq7000::uart::Registers::new_mmio_fixed_0() },
        }
    }

    pub unsafe fn new_fixed_1() -> XPSUart {
        XPSUart {
            regs: unsafe { zynq7000::uart::Registers::new_mmio_fixed_1() },
        }
    }

    pub fn putc(&mut self, ch: char) {
        loop {
            let sts = self.regs.read_status();
            if !sts.tx_full() {
                break;
            } else {
                core::hint::spin_loop();
            }
        }

        let d = zynq7000::uart::Fifo::new_with_raw_value(ch as u32);
        self.regs.write_fifo(d);
    }

    pub fn write(&mut self, msg: &str) {
        for ch in msg.chars() {
            if ch == LF {
                self.putc(CR);
            }
            self.putc(ch);
        }
    }
}

impl core::fmt::Write for XPSUart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s);
        Ok(())
    }
}
