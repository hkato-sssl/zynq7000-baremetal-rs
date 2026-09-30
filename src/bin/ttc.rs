#![no_std]
#![no_main]

use arbitrary_int::u4;
use zynq::print;
use zynq_baremetal as zynq;
use zynq7000::ttc::{RwValue, ClockControl, CounterControl, InterruptControl, Mode, WaveEnable};
use zynq7000_hal::gic::{Interrupt, InterruptGuard};
use zynq7000_hal::ttc::{Ttc, TtcChannel};

const CPU_1X: u32 = 111_111_111;

#[aarch32_rt::irq]
pub extern "C" fn irq_handler() {
    let irq = InterruptGuard::new();

    // Ignore a suprious interrupt.
    if irq.interrupt_info().interrupt() == Interrupt::Spurious {
        return;
    }
}

fn start_ch(ch: &mut TtcChannel) {
    let no = ch.id() as usize;
    let regs = ch.regs_mut();
    let d = InterruptControl::DEFAULT.with_interval(true);
    regs.write_interrupt_enable(no, d).unwrap();
    // Reset and start.
    regs.modify_cnt_ctrl(no, |d| d.with_reset(true).with_disable(false))
        .unwrap();
}

fn init_ch(ch: &mut TtcChannel) {
    let no = ch.id() as usize;
    let regs = ch.regs_mut();
    let d = CounterControl::DEFAULT
        .with_wave_enable_n(WaveEnable::Disable)
        .with_decrementing(true)
        .with_mode(Mode::Interval)
        .with_reset(true)
        .with_disable(true);
    regs.write_cnt_ctrl(no, d).unwrap();

    let d = ClockControl::DEFAULT
        .with_prescaler(u4::new(0x0F))
        .with_prescale_enable(true);
    regs.write_clk_cntr(no, d).unwrap();

    let d = RwValue::DEFAULT.with_value((CPU_1X / 65536) as u16);    // 1s
    regs.write_interval_value(no, d).unwrap();
}

fn read_isr(ch: &mut TtcChannel) -> u32 {
    let no = ch.id() as usize;
    ch.regs_mut().read_interrupt_status(no).unwrap().raw_value()
}

fn ttc_test() {
    let regs = unsafe { zynq7000::ttc::Registers::new_mmio_fixed_0() };
    let mut ttc = Ttc::new(regs).unwrap();
    init_ch(&mut ttc.ch0);
    start_ch(&mut ttc.ch0);
    loop {
        if read_isr(&mut ttc.ch0) != 0 {
            print!("*");
        }
        for _ in 0..100 {
            aarch32_cpu::asm::nop();
        }
    }
}

#[aarch32_rt::entry]
fn main() -> ! {
    zynq::init();

    ttc_test();

    panic!();
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain_secondary() -> ! {
    // There is no work for the secondary core to perform.
    aarch32_cpu::asm::wfi();
    panic!();
}
