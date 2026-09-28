use crate::exception::AbtInfo;
use crate::println;
use aarch32_cpu::register::cpsr::ProcessorMode;
use aarch32_cpu::register::{Dfar, Dfsr, Ifar, Ifsr, SysRegRead};

macro_rules! asm_abort_handler {
    ($handler:ident, $adjust:expr) => {
        core::arch::global_asm!(
            r#"
                .cpu cortex-a9
                .pushsection .text._asm{handler}
                .arm
                .global _asm{handler}
                .type _asm{handler}, %function
                .balign 4
            _asm{handler}:
                // Save the previous context
                sub     lr, lr, #{adjust}
                srsfd   sp, #{abt_mode}
                sub     sp, sp, #{exc_info_size}
                stmia   sp, {{r0-r12, sp, lr}}^
                mov     fp, sp
                bic     sp, fp, #7      // align SP to 8-byte

                // save the other mode's context

                add     r0, fp, #{und_sp_offset}
                cps     #{und_mode}
                str     sp, [r0], #4
                str     lr, [r0], #4
                cps     #{fiq_mode}
                str     sp, [r0], #4
                str     lr, [r0], #4
                cps     #{irq_mode}
                str     sp, [r0], #4
                str     lr, [r0], #4
                cps     #{svc_mode}
                str     sp, [r0], #4
                str     lr, [r0], #4
                cps     #{abt_mode}

                mov     r0, fp
                bl      {handler}

                add     sp, fp, #{pc_offset}
                ldmia   fp, {{r0-r12, sp, lr}}^
                rfefd   sp!

                .size _asm{handler}, . - _asm{handler}
                .popsection
            "#,
            handler = sym $handler,
            adjust = const $adjust as u8,
            abt_mode = const ProcessorMode::Abt as u8,
            und_mode = const ProcessorMode::Und as u8,
            fiq_mode = const ProcessorMode::Fiq as u8,
            irq_mode = const ProcessorMode::Irq as u8,
            svc_mode = const ProcessorMode::Svc as u8,
            exc_info_size = const size_of::<AbtInfo>(),
            und_sp_offset = const core::mem::offset_of!(AbtInfo, und_sp),
            pc_offset = const core::mem::offset_of!(AbtInfo, pc),
        );
    }
}

asm_abort_handler!(_prefetch_abort_handler, 4);
asm_abort_handler!(_data_abort_handler, 8);

fn dump(abt: &AbtInfo) {
    let dfsr = <Dfsr as SysRegRead>::read_raw();
    let dfar = <Dfar as SysRegRead>::read_raw();
    let ifsr = <Ifsr as SysRegRead>::read_raw();
    let ifar = <Ifar as SysRegRead>::read_raw();

    println!("   DFSR: 0x{:08X} DFAR: 0x{:08X}", dfsr, dfar);
    println!("   IFSR: 0x{:08X} IFAR: 0x{:08X}", ifsr, ifar);
    println!("   SPSR: 0x{:08X}", abt.spsr);
    println!("     PC: 0x{:08X}", abt.pc);
    println!("     R0: 0x{:08X}", abt.reg[0]);
    println!("     R1: 0x{:08X}", abt.reg[1]);
    println!("     R2: 0x{:08X}", abt.reg[2]);
    println!("     R3: 0x{:08X}", abt.reg[3],);
    println!("     R4: 0x{:08X}", abt.reg[4]);
    println!("     R5: 0x{:08X}", abt.reg[5]);
    println!("     R6: 0x{:08X}", abt.reg[6]);
    println!("     R7: 0x{:08X}", abt.reg[7]);
    println!("     R8: 0x{:08X}", abt.reg[8]);
    println!("     R9: 0x{:08X}", abt.reg[9]);
    println!("R10(SL): 0x{:08X}", abt.reg[10]);
    println!("R11(FP): 0x{:08X}", abt.reg[11]);
    println!("R12(IP): 0x{:08X}", abt.reg[12]);
    println!(" USR SP: 0x{:08X}  LR: 0x{:08X}", abt.usr_sp, abt.usr_lr);
    println!(" UND SP: 0x{:08X}  LR: 0x{:08X}", abt.und_sp, abt.und_lr);
    println!(" FIQ SP: 0x{:08X}  LR: 0x{:08X}", abt.fiq_sp, abt.fiq_lr);
    println!(" IRQ SP: 0x{:08X}  LR: 0x{:08X}", abt.irq_sp, abt.irq_lr);
    println!(" SVC SP: 0x{:08X}  LR: 0x{:08X}", abt.svc_sp, abt.svc_lr);
}

#[unsafe(no_mangle)]
pub extern "C" fn _data_abort_handler(abt: *const AbtInfo) {
    let abt = unsafe { &*abt };

    println!("\n#### Data Abort ####\n");
    dump(abt);

    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _prefetch_abort_handler(abt: *const AbtInfo) {
    let abt = unsafe { &*abt };

    println!("\n#### Prefetch Abort ####\n");
    dump(abt);

    loop {}
}
