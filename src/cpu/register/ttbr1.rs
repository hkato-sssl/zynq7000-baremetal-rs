use aarch32_cpu::register::{SysReg, SysRegRead, SysRegWrite};

pub struct Ttbr1 {}

impl SysReg for Ttbr1 {
    const CP: u32 = 15;
    const CRN: u32 = 2;
    const OP1: u32 = 0;
    const CRM: u32 = 0;
    const OP2: u32 = 1;
}

impl SysRegRead for Ttbr1 {}

impl SysRegWrite for Ttbr1 {}
