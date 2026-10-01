#![no_std]
#![no_main]

use arbitrary_int::u4;
use core::sync::atomic::{AtomicBool, Ordering};
use zynq::{print, println};
use zynq7000::gic::{DistributorControlRegister, InterfaceControl};
use zynq7000::ttc::{ClockControl, CounterControl, InterruptControl, Mode, RwValue, WaveEnable};
use zynq7000_hal::SpiInterrupt;
use zynq7000_hal::gic::{Interrupt, InterruptGuard, LOWEST_PRIORITY, TargetCpus};
use zynq7000_hal::ttc::{Ttc, TtcChannel};

use zynq_baremetal as zynq;

const CPU_1X: u32 = 111_111_111;

static SIGNAL: AtomicBool = AtomicBool::new(false);

#[aarch32_rt::irq]
pub extern "C" fn irq_handler() {
    let irq = InterruptGuard::new();

    match irq.interrupt_info().interrupt() {
        Interrupt::Spurious => (), // Ignore
        Interrupt::Spi(SpiInterrupt::Ttc00) => isr_ttc(),
        _ => panic!(),
    }
}

fn isr_ttc() {
    // Read ISR to clear the interrupt source.
    let regs = unsafe { zynq7000::ttc::Registers::new_mmio_fixed_0() };
    let mut ttc = Ttc::new(regs).unwrap();
    let _ = read_isr(&mut ttc.ch0);
    SIGNAL.store(true, Ordering::Relaxed);
}

fn init_gic() {
    critical_section::with(|cs| {
        if let Some(gic) = zynq::device::GIC.borrow(cs).borrow_mut().as_mut() {
            gic.set_spi_interrupt_cpu_target_flags(SpiInterrupt::Ttc00, TargetCpus::CPU_0);
            gic.enable_spi_interrupt(SpiInterrupt::Ttc00);
            gic.set_priority_mask(LOWEST_PRIORITY);

            let icr = InterfaceControl::DEFAULT.with_enable_secure(true);
            let dcr = DistributorControlRegister::DEFAULT.with_enable_secure(true);
            gic.update_ctrl_regs(icr, dcr);

            unsafe {
                gic.enable_interrupts();
            }
        }
    });
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

    // Write XTTCPS_CNT_CNTRL.
    let d = CounterControl::DEFAULT
        .with_wave_enable_n(WaveEnable::Disable)
        .with_decrementing(true)
        .with_mode(Mode::Interval)
        .with_reset(true)
        .with_disable(true);
    regs.write_cnt_ctrl(no, d).unwrap();

    // Write XTTCPS_CLK_CNTRL.
    let d = ClockControl::DEFAULT
        .with_prescaler(u4::new(0x0F))
        .with_prescale_enable(true);
    regs.write_clk_cntr(no, d).unwrap();

    // Write XTTCPS_INTERVAL_VAL.
    let d = RwValue::DEFAULT.with_value((CPU_1X / 65536) as u16); // 1s
    regs.write_interval_value(no, d).unwrap();
}

fn read_isr(ch: &mut TtcChannel) -> u32 {
    let no = ch.id() as usize;

    // Read XTTCPS_ISR and clear interrupt sources.
    ch.regs_mut().read_interrupt_status(no).unwrap().raw_value()
}

fn ttc_test() {
    let regs = unsafe { zynq7000::ttc::Registers::new_mmio_fixed_0() };
    let mut ttc = Ttc::new(regs).unwrap();
    init_ch(&mut ttc.ch0);
    init_gic();
    start_ch(&mut ttc.ch0);

    unsafe {
        zynq::debug::gicd::dump();
        zynq::debug::gicc::dump();
    }

    println!("Start TTC Ch0.");
}

#[aarch32_rt::entry]
fn main() -> ! {
    zynq::init();

    ttc_test();

    loop {
        aarch32_cpu::asm::wfi();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain_secondary() -> ! {
    zynq::init();

    unsafe {
        zynq::debug::gicd::dump();
        zynq::debug::gicc::dump();
    }

    loop {
        if SIGNAL.swap(false, Ordering::Relaxed) {
            print!("*");
        }
        for _ in 0..1000 {
            core::hint::spin_loop();
        }
    }
}
