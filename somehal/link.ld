OUTPUT_ARCH(%ARCH%)

ENTRY(_start)


INCLUDE somert.x

SECTIONS
{
    text : ALIGN(PAGE_SIZE) {
        *any_uart-*.o*(.text*)
        *fdt-parser-*.o*(.text*)
        *(.text .text.*)
    } 

    .rodata : ALIGN(PAGE_SIZE) {
        __srodata = .;
        *(.rodata .rodata.*)
        *(.srodata*)
        *(.got .got.*)
    } 
    
    .data : ALIGN(PAGE_SIZE) {
        __erodata = .;
        __srwdata = .;
        
        __sdriver_register = .;
        KEEP(*(.driver.register*))
        __edriver_register = .;


        . = ALIGN(PAGE_SIZE);
        _percpu_load_start = .;
        *(.percpu*)
        . = ALIGN(PAGE_SIZE);
        _percpu_load_end = .;

        *(.data .data.*)

        . = ALIGN(PAGE_SIZE);
        __global_pointer$ = . + 0x800;
        *(.sdata.*)
    } 


    bss (NOLOAD) : ALIGN(PAGE_SIZE)  {
        __kernel_load_end = .;
        __kernel_load_size = . - KIMAGE_VADDR;
        __erwdata = .;

        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        . = ALIGN(PAGE_SIZE);

        __kernel_code_end = .;
    } 


	/DISCARD/ : {
        *(.comment) *(.gnu*) *(.note*) *(.eh_frame*)
    }
} 
