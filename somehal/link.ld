OUTPUT_ARCH(%ARCH%)

ENTRY(_start)

INCLUDE memory.x

SECTIONS
{

    BootText : {
        KEEP(*(.text.boot.header))
        /* KEEP(*(.text.boot*)) */
        *pie_boot-*.o(.text*)
    } > VCODE
    BootData : ALIGN(4K) {
        /* KEEP(*(.data.boot.table)) */
        /* KEEP(*(.data.boot*)) */
        *pie_boot-*.o(.rodata* .got* .data* .sdata* .bss* .sbss* COMMON)
    } > VCODE

    text : ALIGN(4K) {
        *any_uart-*.o(.text*)
        *page_table_generic-*.o(.text*)
        *(.text .text.*)
    } > VCODE

    rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
        *(.got .got.*)
    } > VCODE
    
    .data : ALIGN(4K) {
        __srwdata = .;
        *(.data*)
        . = ALIGN(4K);
        __global_pointer$ = . + 0x800;
        *(.sdata.*)
    } > VCODE

    percpu : ALIGN(4K) {
        __kernel_load_size = . - __start_BootText;
        __erwdata = .;

        *(.percpu*)
    } > VCODE

    bss (NOLOAD) : ALIGN(4K)  {
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        . = ALIGN(4K);
    } > VCODE

    __boot_stack_bottom = .;

	/DISCARD/ : {
        *(.comment) *(.gnu*) *(.note*) *(.eh_frame*)
    }
}

