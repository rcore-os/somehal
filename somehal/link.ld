OUTPUT_ARCH(%ARCH%)

INCLUDE kmem_region.x

SECTIONS
{
    . = __kernel_load_vma;

    text : ALIGN(4K) {
        KEEP(*(.loader))
        . = ALIGN(2M);
        KEEP(*(.text.entry))
        *(.text .text.*)
    } 

    rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
        *(.srodata*)
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
        __kernel_load_size = . - __kernel_load_vma;
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
