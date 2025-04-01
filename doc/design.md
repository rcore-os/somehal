# 设计方案

## Boot (开启mmu前)

一些无法使用的语言特性，使用后程序会跑飞，需要使用括号内替代方式或禁止使用：
    
    1. &dyn Trait (vtable)
    2. &str == &str (&str.eq(&str))
    3. &[T;n] 作为 参数传递 ([T;n])




