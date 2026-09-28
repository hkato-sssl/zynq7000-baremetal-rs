const LOCK_KEY: u32 = 0x767B;
const UNLOCK_KEY: u32 = 0xDF0D;

pub fn lock() {
    let mut slcr = unsafe { zynq7000::slcr::Registers::new_mmio_fixed() };

    slcr.write_lock(LOCK_KEY);
}

pub fn unlock() {
    let mut slcr = unsafe { zynq7000::slcr::Registers::new_mmio_fixed() };

    slcr.write_unlock(UNLOCK_KEY);
}

pub fn stop_secondary_processor() {
    let mut reset = unsafe { zynq7000::slcr::reset::ResetControl::new_mmio_fixed() };

    reset.modify_a9_cpu(|reg| reg.with_cpu1_reset(true).with_cpu1_clockstop(true));
}

pub fn start_secondary_processor() {
    let mut reset = unsafe { zynq7000::slcr::reset::ResetControl::new_mmio_fixed() };

    reset.modify_a9_cpu(|reg| reg.with_cpu1_reset(false));
    reset.modify_a9_cpu(|reg| reg.with_cpu1_clockstop(false));
}
