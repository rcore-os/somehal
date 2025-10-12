/*
 * Max avaliable Page Size is 64K, so we set SectionAlignment
 * field of EFI application to 64K.
 */
PECOFF_FILE_ALIGN = 0x200;
PECOFF_SEGMENT_ALIGN = 0x10000;
VMLINUX_LOAD_ADDRESS = 0x9000000000200000;

/* Provide default stack size if not defined */
PROVIDE(STACK_SIZE = 0x4000);

OUTPUT_ARCH(loongarch)
ENTRY(kernel_entry)
PHDRS {
	text PT_LOAD FLAGS(7);	/* RWX */
	note PT_NOTE FLAGS(4);	/* R__ */
}

SECTIONS
{
    . = VMLINUX_LOAD_ADDRESS;
    _text = .;
    .head.text :  { 
        KEEP(*(.head.text)) 
    }
    . = ALIGN(PECOFF_SEGMENT_ALIGN);
    _stext = .;
    .text : {
        *(.text .text.*)
        *(.init.text)
        *(.fixup)
        *(.gnu.warning)
    } :text = 0
    . = ALIGN(PECOFF_SEGMENT_ALIGN);
    _etext = .;
    _sdata = .;
    .rodata : AT(ADDR(.rodata)) { 
        __start_rodata = .; 
        *(.rodata) 
        *(.rodata.*) 
        *(.data.rel.ro*) 
    }
    .got : ALIGN(16) { 
        *(.got) 
    }
    .plt : ALIGN(16) { 
        *(.plt)
    }
    .got.plt : ALIGN(16) { 
        *(.got.plt)
    }
    . = ALIGN(16384); 
    .data : AT(ADDR(.data)) { 
        . = ALIGN(0x00004000); 
        _sdata = .;
        . = ALIGN(4K);
        *(.data .data.*)
        *(.init.data)
        *(.sdata .sdata.*)
    }
    .rela.dyn : ALIGN(8) {
        __rela_dyn_begin = .;
        *(.rela.dyn) 
        *(.rela*)
        __rela_dyn_end = .;
    }
    .relr.dyn : ALIGN(8) {
        __relr_dyn_begin = .;
        *(.relr.dyn)
    __relr_dyn_end = .;
    }
    .data.rel : { 
        *(.data.rel*)
    }
    . = ALIGN(8);
    .la_abs : {
        __la_abs_begin = .;
        *(.la_abs)
        __la_abs_end = .;
        }
    .sdata : {
        *(.sdata)
    }
    .edata_padding : { 
        BYTE(0); 
        . = ALIGN(PECOFF_FILE_ALIGN); 
        }
    _edata = .;

    .tdata : {
        . = ALIGN(0x10);
        __kernel_load_end = .;
        _stdata = .;
        *(.tdata .tdata.*)
        _etdata = .;
    }

    .tbss : {
        _stbss = .;
        *(.tbss .tbss.*)
        *(.init.bss)
        *(.tcommon)
        _etbss = .;
    }

    . = ALIGN(4K);
    _edata = .;

    . = ALIGN(0); 
        __bss_start = .; 
    . = ALIGN(0); 
    .sbss :  { 
        *(.dynsbss) *(.sbss) *(.scommon) 
    } 
    . = ALIGN(0x00010000); 
    .bss :  { 
        *(.bss..swapper_pg_dir) 
        . = ALIGN(16384); 
        *(.bss..page_aligned) 
        . = ALIGN(16384); 
        *(.dynbss) 
        *(.bss) 
        *(COMMON) 
        
        /* CPU stacks */
        . = ALIGN(16384);
        __cpu0_stack = .;
        . += STACK_SIZE;
        __cpu0_stack_top = .;
        
        /* Init thread union for kernel stack */
        . = ALIGN(16384);
        INIT_THREAD_UNION = .;
        . += 0x4000;  /* 16KB for init thread stack */
        
        /* Firmware arguments storage */
        FW_ARG0 = .;
        . += 8;
        FW_ARG1 = .;
        . += 8;
        FW_ARG2 = .;
        . += 8;
    } . = ALIGN(8); __bss_stop = .;
    . = ALIGN(PECOFF_SEGMENT_ALIGN);
    _end = .;
    
    __kernel_code_end = .;
    .stab 0 : { *(.stab) } .stabstr 0 : { *(.stabstr) } .stab.excl 0 : { *(.stab.excl) } .stab.exclstr 0 : { *(.stab.exclstr) } .stab.index 0 : { *(.stab.index) } .stab.indexstr 0 : { *(.stab.indexstr) }
    .debug 0 : { *(.debug) } .line 0 : { *(.line) } .debug_srcinfo 0 : { *(.debug_srcinfo) } .debug_sfnames 0 : { *(.debug_sfnames) } .debug_aranges 0 : { *(.debug_aranges) } .debug_pubnames 0 : { *(.debug_pubnames) } .debug_info 0 : { *(.debug_info .gnu.linkonce.wi.*) } .debug_abbrev 0 : { *(.debug_abbrev) } .debug_line 0 : { *(.debug_line) } .debug_frame 0 : { *(.debug_frame) } .debug_str 0 : { *(.debug_str) } .debug_loc 0 : { *(.debug_loc) } .debug_macinfo 0 : { *(.debug_macinfo) } .debug_pubtypes 0 : { *(.debug_pubtypes) } .debug_ranges 0 : { *(.debug_ranges) } .debug_weaknames 0 : { *(.debug_weaknames) } .debug_funcnames 0 : { *(.debug_funcnames) } .debug_typenames 0 : { *(.debug_typenames) } .debug_varnames 0 : { *(.debug_varnames) } .debug_gnu_pubnames 0 : { *(.debug_gnu_pubnames) } .debug_gnu_pubtypes 0 : { *(.debug_gnu_pubtypes) } .debug_types 0 : { *(.debug_types) } .debug_addr 0 : { *(.debug_addr) } .debug_line_str 0 : { *(.debug_line_str) } .debug_loclists 0 : { *(.debug_loclists) } .debug_macro 0 : { *(.debug_macro) } .debug_names 0 : { *(.debug_names) } .debug_rnglists 0 : { *(.debug_rnglists) } .debug_str_offsets 0 : { *(.debug_str_offsets) }
    .comment 0 : { *(.comment) } .symtab 0 : { *(.symtab) } .strtab 0 : { *(.strtab) } .shstrtab 0 : { *(.shstrtab) }
    _kernel_entry = ABSOLUTE(kernel_entry & 0xffffffffffff);
    _kernel_asize = ABSOLUTE(_end - _text);
    _kernel_fsize = ABSOLUTE(_edata - _text);
    _kernel_vsize = ABSOLUTE(_end - _sdata);
    _kernel_rsize = ABSOLUTE(_edata - _sdata);
    .gptab.sdata : {
        *(.gptab.data)
        *(.gptab.sdata)
    }
    .gptab.sbss : {
        *(.gptab.bss)
        *(.gptab.sbss)
    }
    /DISCARD/ : { 
        *(.exit.text) 
        *(.text.exit) 
        *(.exit.data .exit.data.*) 
        *(.fini_array .fini_array.*) 
        *(.dtors .dtors.*) 
        *(.exitcall.exit) 
        *(.discard) 
        *(.discard.*) 
        *(.export_symbol) 
        *(.no_trim_symbol) 
        *(.modinfo) 
        *(.gnu.version*) }
    /DISCARD/ : {
        *(.dynamic .dynsym .dynstr .hash .gnu.hash)
        *(.gnu.attributes)
        *(.options)
        *(.eh_frame)
    }
}
