use core::fmt::Write;

const CR: char = '\r';
const LF: char = '\n';

#[bitbybit::bitfield(u32, debug, forbid_overlaps, defmt_bitfields(feature = "defmt"))]
pub struct Control {
    #[bit(4, w)]
    enable_intr: bool,
    #[bit(1, w)]
    rst_rx_fifo: bool,
    #[bit(0, w)]
    rst_tx_fifo: bool,
}

#[bitbybit::bitfield(u32, debug, forbid_overlaps, defmt_bitfields(feature = "defmt"))]
pub struct Status {
    #[bit(7, r)]
    parity_error: bool,
    #[bit(6, r)]
    frame_error: bool,
    #[bit(5, r)]
    overrun_eror: bool,
    #[bit(4, r)]
    intr_enabled: bool,
    #[bit(3, r)]
    tx_fifo_full: bool,
    #[bit(2, r)]
    tx_fifo_empty: bool,
    #[bit(1, r)]
    rx_fifo_full: bool,
    #[bit(0, r)]
    rx_fifo_valid_data: bool,
}

#[derive(derive_mmio::Mmio)]
#[repr(C)]
pub struct Registers {
    #[mmio(Read)]
    pub rx_fifo: u32,
    #[mmio(Write)]
    pub tx_fifo: u32,
    #[mmio(Read)]
    pub stat: Status,
    #[mmio(Write)]
    pub ctrl: Control,
}

pub struct UartLite {
    regs: MmioRegisters<'static>,
}

impl UartLite {
    pub unsafe fn new(reg_base: usize) -> UartLite {
        UartLite {
            regs: unsafe { Registers::new_mmio_at(reg_base) },
        }
    }

    pub fn putc(&mut self, ch: char) {
        loop {
            let sts = self.regs.read_stat();
            if !sts.tx_fifo_full() {
                break;
            } else {
                core::hint::spin_loop();
            }
        }

        self.regs.write_tx_fifo(ch as u32);
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

impl Write for UartLite {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s);
        Ok(())
    }
}
