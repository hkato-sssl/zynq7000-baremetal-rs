#[cfg(all(
    feature = "uart-console-shared",
    not(feature = "uart-console-exclusive")
))]
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let r = critical_section::with(|cs| {
            if let Some(uart) = $crate::device::XPSUART1.borrow(cs).borrow_mut().as_mut() {
                let _ = Write::write_fmt(uart, format_args!($($arg)*));
            }
        });
        r
    }};
}

#[cfg(all(
    feature = "uart-console-exclusive",
    not(feature = "uart-console-shared")
))]
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let r = aarch32_cpu::asm::core_id();
        if r == 0 {
            let _ = critical_section::with(|cs| {
                if let Some(uart) = $crate::pl::UARTLITE0.borrow(cs).borrow_mut().as_mut() {
                    let _ = Write::write_fmt(uart, format_args!($($arg)*));
                }
            });
        } else {
            let _ = critical_section::with(|cs| {
                if let Some(uart) = $crate::pl::UARTLITE1.borrow(cs).borrow_mut().as_mut() {
                    let _ = Write::write_fmt(uart, format_args!($($arg)*));
                }
            });
        }
        r
    }};
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*))
    };
}
