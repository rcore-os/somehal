. = {kimage_vaddr};
PAGE_SIZE = {page_size};

EXTERN(__pie_boot_default_secondary);

PROVIDE(__pie_boot_secondary = __pie_boot_default_secondary);

SECTIONS{
    .head.text : {
        _text = .;
        KEEP(*(.head.text))
    }
    .idmap.text : ALIGN(64){
        _idmap_text = .;
        KEEP(*(.idmap.text))
        . = ALIGN(PAGE_SIZE);
        _idmap_text_end = .;
    }
    .text : ALIGN(4K) {
        _stext = .;
        *(.text.boot)
        *(.text .text.*)
        . = ALIGN(4K);
        _etext = .;
    }
    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
        *(.sdata2 .sdata2.*)
    }

    .init_array : ALIGN(0x10) {
        __init_array_start = .;
        *(.init_array .init_array.*)
        __init_array_end = .;
    }
    . = ALIGN(4K);
    _erodata = .;

    .data : ALIGN(4K) {
        _sdata = .;
        . = ALIGN(4K);
        *(.data .data.*)
        *(.sdata .sdata.*)
        *(.got .got.*)
    }

    .tdata : ALIGN(0x10) {
        __kernel_load_end = .;
        _stdata = .;
        *(.tdata .tdata.*)
        _etdata = .;
    }

    .tbss : ALIGN(0x10) {
        _stbss = .;
        *(.tbss .tbss.*)
        *(.tcommon)
        _etbss = .;
    }

    . = ALIGN(4K);
    _edata = .;

    .bss : AT(.) ALIGN(4K) {
        __bss_start = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        . = ALIGN(4K);
        __bss_stop = .;
    }

    __kernel_code_end = .;

    /DISCARD/ : {
        *(.comment) *(.gnu*) *(.note*) *(.eh_frame*)
    }
}

SECTIONS{ 
    .boot_loader  : {
        KEEP(*(.boot_loader))
    }
} INSERT BEFORE .data

ASSERT(PAGE_SIZE != 0x0, "PAGE_SIZE must be defined");