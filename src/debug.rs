pub mod rgb {
    use crate::driver::axi_gpio::AxiGpio;
    use crate::pl::register::PL_GPIO1_BASE_ADDR;

    fn rgb(r: bool, g: bool, b: bool) -> u32 {
        let mut d: u32;
        d = u32::from(r) << 2;
        d |= u32::from(g) << 1;
        d |= u32::from(b);
        d
    }

    pub unsafe fn set(no: u32, r: bool, g: bool, b: bool) {
        let mut gpio = unsafe { AxiGpio::new(PL_GPIO1_BASE_ADDR) };
        let d = rgb(r, g, b);
        match no {
            0 => gpio.set2(d),
            1 => gpio.set2(d << 3),
            _ => (),
        }
    }

    pub unsafe fn clear(no: u32, r: bool, g: bool, b: bool) {
        let mut gpio = unsafe { AxiGpio::new(PL_GPIO1_BASE_ADDR) };
        let d = rgb(r, g, b);
        match no {
            0 => gpio.clear2(d),
            1 => gpio.clear2(d << 3),
            _ => (),
        }
    }
}

pub mod gicc {
    pub unsafe fn dump() {
        use crate::println;
        let gicc = unsafe { zynq7000::gic::CpuInterfaceRegisters::new_mmio_fixed() };
        println!("#### GIC CPU#{} Interface Registers ####", aarch32_cpu::asm::core_id());
        println!("{:>13}: {:08X}", "ICCICR", gicc.read_icr().raw_value());
        println!("{:>13}: {:08X}", "ICCPMR", gicc.read_pmr().raw_value());
        println!("{:>13}: {:08X}", "ICCBPR", gicc.read_bpr());
        println!("{:>13}: {:08X}", "ICCIAR", gicc.read_iar().raw_value());
        println!("{:>13}: {:08X}", "ICCRPR", gicc.read_rpr().raw_value());
        println!("{:>13}: {:08X}", "ICCHPIR", gicc.read_hpir().raw_value());
        println!("{:>13}: {:08X}", "ICCAPBR", gicc.read_abpr());
        println!("{:>13}: {:08X}", "ICCIIDR", gicc.read_iidr());
        println!();
    }
}

pub mod gicd {
    macro_rules! gicd_array_len {
        ($array:ident) => {{
            fn len<T, U, const N: usize>(_: fn(&T) -> &[U; N]) -> usize {
                N
            }
            len(|t: &zynq7000::gic::DistributorRegisters| &t.$array)
        }};
    }

    macro_rules! gicd_dump_array {
        ($width:expr, $label:expr, $gicd:expr, $array:ident) => {{
            use core::fmt::Write;
            use heapless::String;
            for i in 0..gicd_array_len!($array) {
                let mut label: String<32> = String::new();
                write!(label, concat!($label, "{}"), i).unwrap();
                paste::paste! {
                    println!(concat!("{:>", stringify!($width), "}: {:08X}"), label, $gicd.[<read_ $array>](i).unwrap());
                }
            }
            ()
        }};
        ($width:expr, $label:expr, $gicd:expr, $array:ident, $func:ident) => {{
            use core::fmt::Write;
            use heapless::String;
            for i in 0..gicd_array_len!($array) {
                let mut label: String<32> = String::new();
                write!(label, concat!($label, "{}"), i).unwrap();
                paste::paste! {
                    println!(concat!("{:>", stringify!($width), "}: {:08X}"), label, $gicd.[<read_ $array>](i).unwrap().$func());
                }
            }
            ()
        }};
    }

    pub unsafe fn dump() {
        use crate::println;
        let gicd = unsafe { zynq7000::gic::DistributorRegisters::new_mmio_fixed() };
        println!("#### GIC Distoributor Registers ####");
        println!("{:>13}: {:08X}", "ICDDCR", gicd.read_dcr().raw_value());
        println!("{:>13}: {:08X}", "ICDICTR", gicd.read_ictr().raw_value());
        println!("{:>13}: {:08X}", "ICDIIDR", gicd.read_iidr());
        gicd_dump_array!(13, "ICDISR", gicd, isr);
        gicd_dump_array!(13, "ICDISER", gicd, iser);
        gicd_dump_array!(13, "ICDISPR", gicd, ispr);
        gicd_dump_array!(13, "ICDABR", gicd, abr);
        gicd_dump_array!(13, "ICDIPR", gicd, ipr);
        gicd_dump_array!(13, "ICDIPTR_SGI", gicd, iptr_sgi, raw_value);
        gicd_dump_array!(13, "ICDIPTR_PPI", gicd, iptr_ppi, raw_value);
        gicd_dump_array!(13, "ICDIPTR_SPI", gicd, iptr_spi, raw_value);
        gicd_dump_array!(13, "ICDIPTR_SPI", gicd, iptr_spi, raw_value);
        println!("{:>13}: {:08X}", "ICDICFR0", gicd.read_icfr_0_sgi());
        println!("{:>13}: {:08X}", "ICDICFR1", gicd.read_icfr_1_ppi());
        println!("{:>13}: {:08X}", "ICDICFR2", gicd.read_icfr_2_spi());
        println!("{:>13}: {:08X}", "ICDICFR3", gicd.read_icfr_3_spi());
        println!("{:>13}: {:08X}", "ICDICFR4", gicd.read_icfr_4_spi());
        println!("{:>13}: {:08X}", "ICDICFR5", gicd.read_icfr_5_spi());
        println!("{:>13}: {:08X}", "PPI_STATUS", gicd.read_ppi_status());
        println!("{:>13}: {:08X}", "SPI_STATUS0", gicd.read_spi_status_0());
        println!("{:>13}: {:08X}", "SPI_STATUS1", gicd.read_spi_status_1());
        println!();
    }
}
