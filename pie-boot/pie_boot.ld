SECTIONS{
    . = KIMAGE_VADDR;
    .boot.text : {
        __start_boot_text = .;
        KEEP(*(.text.boot.header))
        KEEP(*pie_boot-*.o*(.text*))
        *any_uart-*.o*(.text*)
        *fdt_parser-*.o*(.text*)
        __stop_boot_text = .;
    } 
    .boot.data : ALIGN(PAGE_SIZE) {
        __start_boot_data = .;
        *pie_boot-*.o*(.rodata* .srodata*  .got* .data* .sdata* .tdata*)
        __stop_boot_data = .;
         __start_boot_bss = .;
        *pie_boot-*.o*(.bss* .sbss* .tbss* COMMON)
        __stop_boot_bss = .;
    }

}

