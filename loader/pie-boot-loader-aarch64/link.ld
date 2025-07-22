ENTRY(_start)

__page_size = {PAGE_SIZE};

SECTIONS
{
    . = 0xF00000000;
    .text : ALIGN(__page_size) {
        _stext  = .;
        *(.text.init)
        *(.text .text.*)
    } 

    .rodata : ALIGN(__page_size) {
        *(.rodata .rodata.*)
        *(.srodata*)
        *(.got .got.*)
    } 
    
    .data : ALIGN(__page_size) {

        *(.data .data.*)
        . = ALIGN(__page_size);
        *(.sdata.*)
    } 

    _BSS : ALIGN(__page_size)  {
        __start_boot_bss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        . = ALIGN(__page_size);
        __stop_boot_bss = .;
        *(.stack);
        __stack_top = .;
    } 
    .rela.dyn : {
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

