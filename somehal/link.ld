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
    } 
    
    .data : ALIGN(__page_size) {
        __erodata = .;
        __srwdata = .;
        
        __sdriver_register = .;
        KEEP(*(.driver.register*))
        __edriver_register = .;


        . = ALIGN(__page_size);
        __start_percpu = .;
        *(.data.percpu*)
        . = ALIGN(__page_size);
        __stop_percpu = .;

        *(.data .data.*)

        . = ALIGN(__page_size);
        __global_pointer$ = . + 0x800;
        *(.sdata.*)
    } 


    bss (NOLOAD) : ALIGN(__page_size)  {
        __kernel_load_size = . - __kernel_load_vma;
        __erwdata = .;

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
