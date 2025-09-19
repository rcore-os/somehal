ENTRY(_start)

__page_size = {PAGE_SIZE};

SECTIONS
{
    . = 0x0;
    .text : ALIGN(__page_size) {
        _stext  = .;
        *(.text.init)
        *(.text .text.*)
    } 

    .rodata : ALIGN(16) {
        *(.rodata .rodata.*)
        *(.srodata*)
        *(.got .got.*)
    } 
    
    .data : ALIGN(16) {

        *(.data .data.*)
        . = ALIGN(16);
        *(.sdata.*)
    } 

    _BSS : ALIGN(16)  {
        __start_boot_bss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        __stop_boot_bss = .;
    } 
    .rela.dyn : ALIGN(16) {
        __rela_dyn_start = .;
        *(.rela .rela*)
        __rela_dyn_end = .;
    }
    _end = .;

	/DISCARD/ : {
        *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) 
        *(.hash)
		*(.interp)
    }
} 

