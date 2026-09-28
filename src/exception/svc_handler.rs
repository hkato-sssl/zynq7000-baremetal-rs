use crate::exception::SvcRequest;
use crate::println;

core::arch::global_asm!(
    r#"
        .cpu cortex-a9
        .pushsection .text._asm_svc_handler
        .arm
        .global _asm_svc_handler
        .type _asm_svc_handler, %function
        .balign 4
    _asm_svc_handler:
        // Save the previous context
        sub     sp, sp, #({svc_context_size} - 4)
        stmia   sp, {{r0-r12, sp, lr}}^
        sub     fp, sp, #4
        bic     sp, fp, #7

        mrs     r0, spsr
        tst     r0, #(1 << 5)   // Test SPSR.T
        bne     from_thumb_code

    from_arm_code:
        sub     lr, lr, #4
        ldr     r1, [lr]
        bic     r1, r1, #0xFF000000
        b       save_rest_context
    
    from_thumb_code:
        sub     lr, lr, #2
        ldrh    r1, [lr]
        and     r1, r1, #0x000000FF

    save_rest_context:
        str     r1, [fp]
        str     lr, [fp, #{pc_offset}]
        str     r0, [fp, #{spsr_offset}]

        mov     r0, fp
        bl      _svc_handler

        add     sp, fp, #{pc_offset}
        ldmia   fp, {{r0-r12, sp, lr}}^
        rfefd   sp!

        .size _asm_svc_handler, . - _asm_svc_handler
        .popsection
    "#,
    svc_context_size = const size_of::<SvcRequest>(),
    pc_offset = const core::mem::offset_of!(SvcRequest, pc),
    spsr_offset = const core::mem::offset_of!(SvcRequest, spsr),
);

fn dump(svc: &SvcRequest) {
    println!("    arg: 0x{:08X}", svc.arg);
    println!("   SPSR: 0x{:08X}", svc.spsr);
    println!("     PC: 0x{:08X}", svc.pc);
    println!("     R0: 0x{:08X}", svc.reg[0]);
    println!("     R1: 0x{:08X}", svc.reg[1]);
    println!("     R2: 0x{:08X}", svc.reg[2]);
    println!("     R3: 0x{:08X}", svc.reg[3],);
    println!("     R4: 0x{:08X}", svc.reg[4]);
    println!("     R5: 0x{:08X}", svc.reg[5]);
    println!("     R6: 0x{:08X}", svc.reg[6]);
    println!("     R7: 0x{:08X}", svc.reg[7]);
    println!("     R8: 0x{:08X}", svc.reg[8]);
    println!("     R9: 0x{:08X}", svc.reg[9]);
    println!("R10(SL): 0x{:08X}", svc.reg[10]);
    println!("R11(FP): 0x{:08X}", svc.reg[11]);
    println!("R12(IP): 0x{:08X}", svc.reg[12]);
    println!(" USR_SP: 0x{:08X}", svc.sp);
    println!(" USR_LR: 0x{:08X}", svc.lr);
}

#[unsafe(no_mangle)]
pub extern "C" fn _svc_handler(svc: *mut SvcRequest) {
    let svc = unsafe { &mut *svc };

    println!("\n#### SVC Exception ####\n");
    dump(svc);

    loop {}
}
