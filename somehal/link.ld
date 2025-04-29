OUTPUT_ARCH(%ARCH%)

ENTRY(_start)


INCLUDE pie_boot.x

SECTIONS
{
    text : ALIGN(__page_size) {
        *any_uart-*.o*(.text*)
        *fdt-parser-*.o*(.text*)
        *(.text .text.*)
    } 

    .rodata : ALIGN(__page_size) {
        __srodata = .;
        *(.rodata .rodata.*)
        *(.srodata*)
        *(.got .got.*)

        __sdriver_register = .;
        *(.driver.register*)
        __edriver_register = .;
    } 
    
    .data : ALIGN(__page_size) {
        __erodata = .;
        __srwdata = .;
        *(.data*)
        . = ALIGN(__page_size);
        __global_pointer$ = . + 0x800;
        *(.sdata.*)
    } 
    percpu : ALIGN(__page_size) {
        __kernel_load_size = . - __kernel_load_vma;
        __erwdata = .;

        *(.percpu*)
    } 

    bss (NOLOAD) : ALIGN(__page_size)  {
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        . = ALIGN(__page_size);

        __kernel_code_end = .;
    } 


	/DISCARD/ : {
        *(.comment) *(.gnu*) *(.note*) *(.eh_frame*)
    }
} 
