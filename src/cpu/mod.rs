pub mod register;

use aarch32_cpu::register::{Cpacr, SysRegWrite, Vbar};

unsafe extern "C" {
    static _vector_table: u8;
}

pub enum CpacrAccessPermission {
    Deninde = 0b00,
    Privileged = 0b01,
    Reserved = 0b10,
    PrivilegedAndUser = 0b11,
}

fn enable_fpu() {
    let cp11 = (CpacrAccessPermission::PrivilegedAndUser as u32) << 22;
    let cp10 = (CpacrAccessPermission::PrivilegedAndUser as u32) << 20;
    let d = cp11 | cp10;

    unsafe {
        <Cpacr as SysRegWrite>::write_raw(d);
        aarch32_cpu::asm::isb();

        core::arch::asm!(
            r#"
                mov     r0, #(1 << 30)  // Set FPEXC.EN
                vmsr    fpexc, r0
                mov     r0, #0
                vmsr    fpscr, r0
            "#
        );
    }
}

pub fn init() {
    unsafe {
        let vec = core::ptr::addr_of!(_vector_table);
        <Vbar as SysRegWrite>::write_raw(vec as u32);
        aarch32_cpu::asm::isb();
    }

    enable_fpu();
}
