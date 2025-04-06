OUTPUT_ARCH(%ARCH%)

ENTRY(_start)

SECTIONS
{
    . = %KERNEL_VADDR% ;
    BootText : {
        KEEP(*(.text.boot.header))
        KEEP(*(.text.boot*))
    }
    BootData : ALIGN(4K) {
        KEEP(*(.data.boot*))
    }

    text : ALIGN(4K) {
        *(.text .text.*)
    } 

    rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
        *(.got .got.*)
    } 
   
    data : ALIGN(4K) {
        *(.data*)
        *(.sdata .sdata.*)
    }

    percpu : ALIGN(4K) {
        *(.percpu .percpu*)
    }

    __ekernel_load = .;
    __kernel_size = . - __start_BootText;

    bss (NOLOAD) : ALIGN(4K)  {
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        . = ALIGN(4K);
    }

    __stack_bottom = .;
	/DISCARD/ : {
        *(.comment) *(.gnu*) *(.note*) *(.eh_frame*)
    }
}
