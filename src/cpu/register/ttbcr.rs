use aarch32_cpu::register::{SysReg, SysRegRead, SysRegWrite};

pub struct Ttbcr {}

impl SysReg for Ttbcr {
    const CP: u32 = 15;
    const CRN: u32 = 2;
    const OP1: u32 = 0;
    const CRM: u32 = 0;
    const OP2: u32 = 2;
}

impl SysRegRead for Ttbcr {}

impl SysRegWrite for Ttbcr {}
