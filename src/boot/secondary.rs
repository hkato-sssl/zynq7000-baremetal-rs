use crate::driver::slcr;

pub fn start_secondary_processor() {
    aarch32_cpu::interrupt::free(|| {
        slcr::unlock();
        slcr::stop_secondary_processor();
        setup_secondary_boot_vector();
        slcr::start_secondary_processor();
        slcr::lock();
    });
}

#[cfg(target_arch = "arm")]
core::arch::global_asm!(
    r#"
        .pushsection .text.secondary_boot_vector
        .arm
        .type __secondary_boot_vector, %function
        .global __secondary_boot_vector
        .global __secondary_boot_vector_end    
    __secondary_boot_vector:
        ldr     pc, 0f
    0:
        .word   _start
    __secondary_boot_vector_end:

        .size   __secondary_boot_vector, . - __secondary_boot_vector
        .popsection
    "#
);

unsafe extern "C" {
    static __secondary_boot_vector: u8;
    static __secondary_boot_vector_end: u8;
}

const VECTOR_ADDR: usize = 0;

fn setup_secondary_boot_vector() {
    let vec_start = core::ptr::addr_of!(__secondary_boot_vector) as usize;
    let vec_end = core::ptr::addr_of!(__secondary_boot_vector_end) as usize;
    let vec_sz = vec_end - vec_start;

    unsafe {
        for i in 0..vec_sz {
            let src = vec_start + i;
            let dst = VECTOR_ADDR + i;
            let d = core::ptr::read_volatile(src as *mut u8);
            core::ptr::write_volatile(dst as *mut u8, d);
        }
    }

    crate::cache::d_clean(VECTOR_ADDR as u32);
}
