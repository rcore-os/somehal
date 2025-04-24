/* INCLUDE kmem_region.x */

SECTIONS{
    /* . = 0x40200000; */
    .text : {
        __start_boot_text = .;
        KEEP(*(.text.boot.header))
        KEEP(*pie_boot-*.o(.text*))
        __stop_boot_text = .;
    } 
    .data : ALIGN(4K) {
        __start_boot_data = .;
        *(.data.table*)
        *(.rodata* .srodata* .got* .data* .sdata*)
        __stop_boot_data = .;
    }
    .bss : ALIGN(4K) {
         __start_boot_bss = .;
        *(.bss* .sbss* COMMON)
        . = ALIGN(16);
        *(.bss.stack*) 
        __stop_boot_bss = .;
    }
    __kernel_code_end = .;
    __kernel_load_size = . - __start_boot_text;

    /DISCARD/ : {
        *(.comment) *(.gnu*) *(.note*) *(.eh_frame*)
    }
}