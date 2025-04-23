OUTPUT_ARCH(%ARCH%)

ENTRY(_start)

/* from kmem crate */
INCLUDE memory.x

SECTIONS
{
    . = __kernel_load_vma;
    
    BootText : {
        KEEP(*(.text.boot.header))
        KEEP(*pie_boot-*.o(.text*))
    } 
    BootData : ALIGN(4K) {
        *pie_boot-*.o(.rodata* .got* .data* .sdata*)
        __start_BootBss = .;
        *pie_boot-*.o(.bss* .sbss* COMMON)
        __stop_BootBss = .;
    } 
    

    text : ALIGN(4K) {
        *any_uart-*.o(.text*)
        *page_table_generic-*.o(.text*)
        *fdt-parser-*.o(.text*)
        *(.text .text.*)
    } 

    rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
        *(.got .got.*)
    } 
    
    .data : ALIGN(4K) {
        __srwdata = .;
        *(.data*)
        . = ALIGN(4K);
        __global_pointer$ = . + 0x800;
        *(.sdata.*)
    } 

    percpu : ALIGN(4K) {
        __kernel_load_size = . - __start_BootText;
        __erwdata = .;

        *(.percpu*)
    } 

    bss (NOLOAD) : ALIGN(4K)  {
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        . = ALIGN(4K);

        __kernel_code_end = .;
    } 


	/DISCARD/ : {
        *(.comment) *(.gnu*) *(.note*) *(.eh_frame*)
    }
} 
