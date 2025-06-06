INCLUDE kmem_region.x

SECTIONS{
    . = __kernel_load_vma;
    .boot.text : {
        __start_boot_text = .;
        KEEP(*(.text.boot.header))
        KEEP(*pie_boot-*.o*(.text*))
        __stop_boot_text = .;
    } 
    .boot.data : ALIGN(__page_size) {
        __start_boot_data = .;
        *pie_boot-*.o*(.rodata* .srodata*  .got* .data* .sdata* .tdata*)
        __stop_boot_data = .;
         __start_boot_bss = .;
        *pie_boot-*.o*(.bss* .sbss* .tbss* COMMON)
        __stop_boot_bss = .;
    }

}

