use aarch32_cpu::register::cpsr::ProcessorMode;

const SCTLR_RESERVED: u32 = 0x00C50078;
const SCTLR_IZ: u32 = SCTLR_RESERVED | (1 << 12) | (1 << 11);

core::arch::global_asm!(
    r#"
        .fpu vfp2
        .cpu cortex-a9
        .pushsection .text._start
        .arm

        .global _start
        .type _start, %function
    _start:
        cpsid   iaf, #{sys_mode}
        bl      init_sp

        // Disable MMU and L1 cache
        movw    r0, #:lower16:{initial_sctlr}
        movt    r0, #:upper16:{initial_sctlr}
        dsb
        isb
        mcr     p15, 0, r0, c1, c0, 0   // Write SCTLR
        isb

        mov     r0, #0        
        mcr     p15, 0, r0, c1, c0, 1   // Write ACTLR
        isb

        // Invalidate all TLB
        mcr     p15, 0, r0, c8, c7, 0   // TLBIALL
        dsb
        isb

        // Invalidate L1 I-$ and branch predictor
        mcr     p15, 0, r0, c7, c5, 0   // ICIALLU
        dsb
        isb

        // Enable L1 I-$ and branch predictor
        movw    r0, #:lower16:{sctlr_iz}
        movt    r0, #:upper16:{sctlr_iz}
        mcr     p15, 0, r0, c1, c0, 0   // Write SCTLR
        isb

        mrc     p15, 0, r0, c0, c0, 5   // Read MPIDR
        ands    r0, r0, #0xFF
        bne     boot_secondary

        bl      clean_and_invalidate_all_l1_d_cache
        bl      clear_bss_section
        bl      kmain
        b       .

    boot_secondary:
        bl      invalidate_all_l1_d_cache
        bl      kmain_secondary
        b       .

    clean_and_invalidate_all_l1_d_cache:
        mov     r0, #0
        mcr     p15, 2, r0, c0, c0, 0   // Write CSSELR
        isb 
        mrc     p15, 1, r0, c0, c0, 0   // Read CCSIDR
        movw    r1, #0x003F
        and     r1, r1, r0, lsr # 3
        clz     r2, r1
        movw    r1, #1
        lsl     r1, r1, r2
        and     r2, r0, #0x07
        add     r2, r2, #4
        movw    ip, #0x7FFF
        and     r0, ip, r0, lsr #13
        dmb
        // r0: <# of sets> - 1
        // r1: way increment value
        // r2: shift count for sets
    0:
        lsl     r3, r0, r2
    1:
        mcr     p15, 0, r3, c7, c14, 2  // DCCISW
        adds    r3, r3, r1
        bcc     1b

        subs    r0, r0, #1
        bcs     0b

        dsb
        bx      lr

    invalidate_all_l1_d_cache:
        mov     r0, #0
        mcr     p15, 2, r0, c0, c0, 0   // Write CSSELR
        isb 
        mrc     p15, 1, r0, c0, c0, 0   // Read CCSIDR
        movw    r1, #0x003F
        and     r1, r1, r0, lsr # 3
        clz     r2, r1
        movw    r1, #1
        lsl     r1, r1, r2
        and     r2, r0, #0x07
        add     r2, r2, #4
        movw    ip, #0x7FFF
        and     r0, ip, r0, lsr #13
        dmb
        // r0: <# of sets> - 1
        // r1: way increment value
        // r2: shift count for sets
    0:
        lsl     r3, r0, r2
    1:
        mcr     p15, 0, r3, c7, c6, 2   // DCISW
        adds    r3, r3, r1
        bcc     1b

        subs    r0, r0, #1
        bcs     0b

        dsb
        bx      lr

    clear_bss_section:
        // This function requires __sbss and __ebss to be 4-byte aligned.
        ldr     r0, =__sbss
        ldr     r1, =__ebss
        subs    r1, r1, r0
        beq     1f

        sub     r1, r1, #4
        mov     r2, #0
    0:
        str     r2, [r0, r1]
        subs    r1, r1, #4
        bcs     0b
    1:
        bx      lr

        .size _start, . - _start
        .popsection
    "#,
    initial_sctlr = const SCTLR_RESERVED,
    sctlr_iz = const SCTLR_IZ,
    sys_mode = const ProcessorMode::Sys as u8,
);

core::arch::global_asm!(
    r#"
        .fpu vfp2
        .cpu cortex-a9
        .pushsection .text.init_sp
        .arm

        .type init_sp, %function
    init_sp:
        mrc     p15, 0, r0, c0, c0, 5   // Read MPIDR
        and     r0, r0, #0xFF
        add     r0, r0, #1

        cps     #{und_mode}
        ldr     sp, =_und_stack_low_end
        ldr     r1, =_und_stack_size
        mla     sp, r0, r1, sp

        cps     #{abt_mode}
        ldr     sp, =_abt_stack_low_end
        ldr     r1, =_abt_stack_size
        mla     sp, r0, r1, sp

        cps     #{irq_mode}
        ldr     sp, =_irq_stack_low_end
        ldr     r1, =_irq_stack_size
        mla     sp, r0, r1, sp

        cps     #{fiq_mode}
        ldr     sp, =_fiq_stack_low_end
        ldr     r1, =_fiq_stack_size
        mla     sp, r0, r1, sp

        cps     #{svc_mode}
        ldr     sp, =_svc_stack_low_end
        ldr     r1, =_svc_stack_size
        mla     sp, r0, r1, sp

        cps     #{sys_mode}
        ldr     sp, =_sys_stack_low_end
        ldr     r1, =_sys_stack_size
        mla     sp, r0, r1, sp

        bx      lr

        .size init_sp, . - init_sp
        .popsection
    "#,
    sys_mode = const ProcessorMode::Sys as u8,
    und_mode = const ProcessorMode::Und as u8,
    abt_mode = const ProcessorMode::Abt as u8,
    irq_mode = const ProcessorMode::Irq as u8,
    fiq_mode = const ProcessorMode::Fiq as u8,
    svc_mode = const ProcessorMode::Svc as u8,
);
