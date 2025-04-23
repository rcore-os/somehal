cfg_if::cfg_if! {
    if #[cfg(feature = "vm")]{
        mod table_el2;
        pub use table_el2::*;
    }else{
        mod table_el1;
        pub use table_el1::*;
    }
}
