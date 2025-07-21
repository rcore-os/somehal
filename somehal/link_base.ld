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
}

SECTIONS{ 
    .boot_loader  : {
        KEEP(*(.boot_loader))
    }
} INSERT BEFORE .data

ASSERT(PAGE_SIZE != 0x0, "PAGE_SIZE must be defined");