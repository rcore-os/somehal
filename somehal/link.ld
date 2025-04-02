OUTPUT_ARCH(%ARCH%)

ENTRY(_start)

SECTIONS
{
    . = %KERNEL_VADDR% ;
    BootText : {
        KEEP(*(.text.boot*))
    }
    BootData : ALIGN(4K) {
        KEEP(*(.data.boot*))
    }

    text : ALIGN(4K) {
        *(.text*);
    } 

    rodata : ALIGN(4K) {
        *(.rodata*)
        *(.got*)
    } 
   
    data : ALIGN(4K) {
        *(.data*)
    }
    __kernel_size = . - __start_BootText;

    bss (NOLOAD) : ALIGN(4K)  {
        *(.bss*)
        *(.sbss*)
        *(COMMON)
        . = ALIGN(4K);
    }

    __stack_bottom = .;
	/DISCARD/ : {
        *(.comment) *(.gnu*) *(.note*) *(.eh_frame*)
    }
}
