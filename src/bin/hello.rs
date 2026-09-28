#![no_std]
#![no_main]

use aarch32_rt::entry;
use zynq::println;
use zynq_baremetal as zynq;

fn uart_test() {
    let core = aarch32_cpu::asm::core_id();

    println!("START#{}", core);

    for i in 0..100000 {
        println!("{}:HELLO#{}", core, i);
        aarch32_cpu::asm::sev();
        aarch32_cpu::asm::wfe();
    }
}

#[entry]
fn main() -> ! {
    zynq::init();

    uart_test();

    panic!();
}

#[unsafe(no_mangle)]
pub extern "C" fn kmain_secondary() -> ! {
    zynq::init();

    uart_test();

    panic!();
}
