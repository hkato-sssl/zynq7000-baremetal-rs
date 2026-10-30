const DACR: u32 = 0xFFFFFFFD; // Domain0: Client, Others: Manager
const ACTLR_SMP_FW: u32 = (1 << 6) | 1;
const SCTLR_IZCM: u32 = (1 << 12) | (1 << 11) | (1 << 2) | 1;

pub(super) fn enable_mmu_and_cache() {
    unsafe {
        core::arch::asm!(
            r#"
                movw    r0, #:lower16:{dacr}
                movt    r0, #:upper16:{dacr}
                mcr     p15, 0, r0, c3, c0, 0   // Write DACR

                movw    r0, #{actlr}
                mcr     p15, 0, r0, c1, c0, 1   // Write ACTLR

                mrc     p15, 0, r0, c1, c0, 0   // Read SCTLR
                movw    r1, #{sctlr_izcm}
                orr     r0, r0, r1

                dsb
                isb
                mcr     p15, 0, r0, c1, c0, 0   // Read SCTLR
                isb

                // Invalidate all TLB
                mcr     p15, 0, r0, c8, c7, 0   // TLBIALL
                dsb
                isb

                // Invalidate L1 I-$ and branch predictor
                mcr     p15, 0, r0, c7, c5, 0   // ICIALLU
                dsb
                isb
            "#,
            dacr = const DACR,
            actlr = const ACTLR_SMP_FW,
            sctlr_izcm = const SCTLR_IZCM,
        );
    }
}
