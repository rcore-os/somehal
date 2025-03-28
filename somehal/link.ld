OUTPUT_ARCH(%ARCH%)

ENTRY(_start)

SECTIONS
{
    . = %KERNEL_VADDR% ;
    _skernel = .;

    .text : ALIGN(4K) {
        
        KEEP(*(.text.init*))
        *(.text*);
    } 

    .rodata : ALIGN(4K) {
        *(.rodata*)
        *(.srodata*)
        *(.sdata2*)
    } 
   
    .data : ALIGN(4K) {
        *(.data*)
        *(.sdata*)
        *(.got*)
    }

    .tdata : ALIGN(0x10) {
        *(.tdata*)
    } 

    _ekernel = .;
    _kernel_size = _ekernel - _skernel;

    .tbss (NOLOAD) : ALIGN(0x10) {
        *(.tbss .tbss.*)
        *(.tcommon)
    }

    .bss (NOLOAD) : ALIGN(4K) {
        *(.bss*)
        *(.sbss*)
        *(COMMON)
        . = ALIGN(4K);
    }

    _stack_bottom = .;
    _stack_top = . + %STACK_SIZE%;
	/DISCARD/ : {
        *(.comment) *(.gnu*) *(.note*) *(.eh_frame*)
    }
}
