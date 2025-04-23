ENTRY (_start)
SECTIONS
{
    . = 1M;
    BootText : {
        KEEP(*(.text.boot.header))
        KEEP(*(.text.boot*))
        KEEP(*pie_boot-*.o(.text*))
    }
    BootData : ALIGN(4K) {
        KEEP(*(.data.boot*))
        *pie_boot-*.o(.rodata* .got* .data* .sdata* .bss* .sbss* COMMON)
    }

    text : ALIGN(4K) {
        *any_uart-*.o(.text*)
        *page_table_generic-*.o(.text*)
        *(.text .text.*)
    } 

    rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
        *(.got .got.*)
    } 
    
    .data : ALIGN(4K) {
        __srwdata = .;
        *(.data*)
        *(.sdata.*)
    }

    percpu : ALIGN(4K) {
        __kernel_load_size = . - __kernel_load_vma;
        __erwdata = .;

        *(.percpu*)
    }

    bss (NOLOAD) : ALIGN(4K)  {
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        . = ALIGN(4K);
    }

	/DISCARD/ : {
        *(.comment) *(.gnu*) *(.note*) *(.eh_frame*)
    }
    __kernel_load_end = LOADADDR(percpu);
}