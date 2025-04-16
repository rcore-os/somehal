OUTPUT_ARCH(%ARCH%)

ENTRY(_start)

SECTIONS
{
    . = %KERNEL_VADDR% ;
    BootText : {
        KEEP(*(.text.boot.header))
        KEEP(*(.text.boot*))
        *pie_boot-*.o(.text*)
    }
    BootData : ALIGN(4K) {
        KEEP(*(.data.boot.table))
        KEEP(*(.data.boot*))
        *pie_boot-*.o(.rodata* .got* .data* .sdata* .bss* .sbss* COMMON)
    }

    text : ALIGN(4K) {
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
    }

    __boot_stack_bottom = .;

	/DISCARD/ : {
        *(.comment) *(.gnu*) *(.note*) *(.eh_frame*)
    }
}
