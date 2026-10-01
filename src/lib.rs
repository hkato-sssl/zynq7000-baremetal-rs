#![no_std]

mod boot;
pub mod cache;
pub mod cpu;
pub mod debug;
pub mod device;
pub mod driver;
pub mod exception;
pub mod macros;
pub mod mmu_ref;
pub mod pl;

pub use mmu_ref as mmu;

#[cfg(target_os = "none")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        debug::rgb::set(aarch32_cpu::asm::core_id(), true, false, false);
    }
    crate::println!("PANIC: {:#?}", info);
    loop {}
}

pub fn init() {
    cpu::init();
    mmu::init();
    mmu::enable_mmu_and_cache();
    if aarch32_cpu::asm::core_id() == 0 {
        cache::l2::init();
    }
    device::init();
    pl::init();

    if aarch32_cpu::asm::core_id() == 0 {
        boot::start_secondary_processor();
    }
}
