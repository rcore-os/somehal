#[macro_export(local_inner_macros)]
macro_rules! sym_lma {
    ($sym:expr) => {{
        #[allow(unused_unsafe)]
        unsafe{
            let out: usize;
            core::arch::asm!(
                "adrp {r}, {s}",
                "add  {r}, {r}, :lo12:{s}",
                r = out(reg) out,
                s = sym $sym,
            );
            out
        }
    }};
}

#[macro_export(local_inner_macros)]
macro_rules! sym_lma_extern {
    ($sym:ident) => {{
        unsafe extern "C" {
            static $sym: u8;
        }
        sym_lma!($sym)
    }};
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::console::__print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    ()=>{
        $crate::console::__print_str("\r\n");
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\r\n", format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! printkv {
    ($key:expr, $($arg:tt)*) => {
        $crate::print!("{:<24}: {}\r\n", $key, format_args!($($arg)*))
    };
}
