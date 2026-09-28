#[derive(derive_mmio::Mmio)]
#[repr(C)]
pub struct Registers {
    pub data: u32,
    pub tri: u32,
    pub data2: u32,
    pub tri2: u32,
}

impl Registers {
    pub unsafe fn new(reg_base: usize) -> MmioRegisters<'static> {
        MmioRegisters {
            ptr: reg_base as *mut Registers,
            phantom: core::marker::PhantomData,
        }
    }
}

pub struct AxiGpio {
    regs: MmioRegisters<'static>,
}

impl AxiGpio {
    pub unsafe fn new(reg_base: usize) -> AxiGpio {
        AxiGpio {
            regs: unsafe { Registers::new(reg_base) },
        }
    }

    pub fn set(&mut self, bits: u32) {
        aarch32_cpu::interrupt::free(|| {
            self.regs.modify_data(|d| d | bits);
        });
    }

    pub fn clear(&mut self, bits: u32) {
        aarch32_cpu::interrupt::free(|| {
            self.regs.modify_data(|d| d & !bits);
        });
    }

    pub fn set2(&mut self, bits: u32) {
        aarch32_cpu::interrupt::free(|| {
            self.regs.modify_data2(|d| d | bits);
        });
    }

    pub fn clear2(&mut self, bits: u32) {
        aarch32_cpu::interrupt::free(|| {
            self.regs.modify_data2(|d| d & !bits);
        });
    }
}
