use crate::lazy_static::LazyStatic;

#[unsafe(link_section = ".data")]
pub static CPU_NUM: LazyStatic<usize> = LazyStatic::with_default(1);
