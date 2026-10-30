use aarch32_cpu::register::{SysReg, SysRegWrite};

/// TLBIALLMVAIS
pub struct TlbIMvaIs;

impl SysReg for TlbIMvaIs {
    const CP: u32 = 15;
    const CRN: u32 = 8;
    const OP1: u32 = 0;
    const CRM: u32 = 3;
    const OP2: u32 = 1;
}

impl SysRegWrite for TlbIMvaIs {}

impl TlbIMvaIs {
    pub fn write(mva: u32) {
        unsafe { <Self as SysRegWrite>::write_raw(mva) }
    }
}
