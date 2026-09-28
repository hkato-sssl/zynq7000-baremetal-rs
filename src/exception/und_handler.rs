use crate::exception::UndInfo;
use crate::println;
use aarch32_cpu::register::cpsr::ProcessorMode;

core::arch::global_asm!(
    r#"
        .cpu cortex-a9
        .pushsection .text._asm_undefined_handler
        .arm
        .global _asm_undefined_handler
        .type _asm_undefined_handler, %function
        .balign 4
    _asm_undefined_handler:
        // Save the user mode context
        sub     sp, sp, #{und_info_size}
        stmia   sp, {{r0-r12, sp, lr}}^
        mov     fp, sp
        bic     sp, sp, #7

        // save the other mode's context
        add     r0, fp, #{abt_sp_offset}
        cps     #{abt_mode}
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
        cps     #{und_mode}

        // Adjust PC
        mrs     r0, spsr
        tst     r0, #(1 << 5)   // Test SPSR.T
        subeq   lr, lr, #4
        subne   lr, lr, #2
        str     lr, [fp, #{pc_offset}]
        str     r0, [fp, #{spsr_offset}]

        mov     r0, fp
        bl      undefined_handler

        add     sp, fp, #{pc_offset}
        ldmia   fp, {{r0-r12, sp, lr}}^
        rfefd   sp!

        .size _asm_undefined_handler, . - _asm_undefined_handler
        .popsection
    "#,
    und_info_size = const size_of::<UndInfo>(),
    abt_mode = const ProcessorMode::Abt as u8,
    fiq_mode = const ProcessorMode::Fiq as u8,
    irq_mode = const ProcessorMode::Irq as u8,
    svc_mode = const ProcessorMode::Svc as u8,
    und_mode = const ProcessorMode::Und as u8,
    abt_sp_offset = const core::mem::offset_of!(UndInfo, abt_sp),
    pc_offset = const core::mem::offset_of!(UndInfo, pc),
    spsr_offset = const core::mem::offset_of!(UndInfo, spsr),
);

fn dump(und: &UndInfo) {
    println!("   SPSR: 0x{:08X}", und.spsr);
    println!("     PC: 0x{:08X}", und.pc);
    println!("     R0: 0x{:08X}", und.reg[0]);
    println!("     R1: 0x{:08X}", und.reg[1]);
    println!("     R2: 0x{:08X}", und.reg[2]);
    println!("     R3: 0x{:08X}", und.reg[3],);
    println!("     R4: 0x{:08X}", und.reg[4]);
    println!("     R5: 0x{:08X}", und.reg[5]);
    println!("     R6: 0x{:08X}", und.reg[6]);
    println!("     R7: 0x{:08X}", und.reg[7]);
    println!("     R8: 0x{:08X}", und.reg[8]);
    println!("     R9: 0x{:08X}", und.reg[9]);
    println!("R10(SL): 0x{:08X}", und.reg[10]);
    println!("R11(FP): 0x{:08X}", und.reg[11]);
    println!("R12(IP): 0x{:08X}", und.reg[12]);
    println!(" USR SP: 0x{:08X}  LR: 0x{:08X}", und.usr_sp, und.usr_lr);
    println!(" ABT SP: 0x{:08X}  LR: 0x{:08X}", und.abt_sp, und.abt_lr);
    println!(" FIQ SP: 0x{:08X}  LR: 0x{:08X}", und.fiq_sp, und.fiq_lr);
    println!(" IRQ SP: 0x{:08X}  LR: 0x{:08X}", und.irq_sp, und.irq_lr);
    println!(" SVC SP: 0x{:08X}  LR: 0x{:08X}", und.svc_sp, und.svc_lr);
}

#[unsafe(no_mangle)]
pub extern "C" fn undefined_handler(und: *mut UndInfo) {
    let und = unsafe { &mut *und };

    println!("\n#### Undefined instruction exception ####\n");
    dump(und);

    loop {}
}
