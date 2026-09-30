pub mod abt_handler;
pub mod svc_handler;
pub mod und_handler;

#[repr(C)]
pub struct AbtInfo {
    pub reg: [u32; 13],
    pub usr_sp: u32,
    pub usr_lr: u32,
    pub und_sp: u32,
    pub und_lr: u32,
    pub fiq_sp: u32,
    pub fiq_lr: u32,
    pub irq_sp: u32,
    pub irq_lr: u32,
    pub svc_sp: u32,
    pub svc_lr: u32,
    pub pc: u32,
    pub spsr: u32,
}

#[repr(C)]
pub struct UndInfo {
    pub reg: [u32; 13],
    pub usr_sp: u32,
    pub usr_lr: u32,
    pub abt_sp: u32,
    pub abt_lr: u32,
    pub fiq_sp: u32,
    pub fiq_lr: u32,
    pub irq_sp: u32,
    pub irq_lr: u32,
    pub svc_sp: u32,
    pub svc_lr: u32,
    pub pc: u32,
    pub spsr: u32,
}

#[repr(C)]
pub struct SvcRequest {
    pub arg: u32,
    pub reg: [u32; 13], // r0-r12
    pub sp: u32,        // sp_usr
    pub lr: u32,        // lr_usr
    pub pc: u32,
    pub spsr: u32,
}
