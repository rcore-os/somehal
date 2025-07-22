use crate::lazy_static::LazyStatic;

pub static CPU_NUM: LazyStatic<usize> = LazyStatic::with_default(1);
