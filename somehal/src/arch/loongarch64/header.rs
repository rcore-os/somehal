use core::arch::naked_asm;

use super::addrspace::{VMLINUX_LOAD_ADDRESS, to_phys};
use crate::common::pe::*;
use pie_boot_macros::start_code;

/// LoongArch64 kernel header implementing functionality similar to
/// Linux arch/loongarch/kernel/head.S _head section
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".head.text")]
pub unsafe extern "C" fn _head() -> ! {
    naked_asm!(
        // EFI header following Linux kernel format
        ".word {dos_signature}",        // "MZ", MS-DOS header
        ".org 0x8",
        ".dword _kernel_entry",               // Kernel entry point (physical address)
        ".dword _kernel_asize",         // Kernel image effective size
        ".quad {phys_link_kaddr}",      // PHYS_LINK_KADDR - Kernel image load offset
        ".org 0x38",                    // 0x20 ~ 0x37 reserved
        ".long {linux_pe_magic}",
        ".long 1f - _head",               // Offset to the PE header

        ".long {image_nt_signature}",               // IMAGE_NT_SIGNATURE

        "1:",
        // COFF header
        ".short {file_machine}",                  // IMAGE_FILE_MACHINE_LOONGARCH64
        ".short 2",                       // NumberOfSections
        ".long 0",                        // TimeDateStamp
        ".long 0",                        // PointerToSymbolTable
        ".long 0",                        // NumberOfSymbols
        ".short 3f - 2f",                 // SizeOfOptionalHeader
        ".short 0x0206",                  // Characteristics

        // Optional header
        "2:",  // optional_header
        ".short 0x020b",                  // IMAGE_NT_OPTIONAL_HDR64_MAGIC
        ".byte 0x02",                     // MajorLinkerVersion
        ".byte 0x14",                     // MinorLinkerVersion
        ".long _etext - 4f",              // SizeOfCode
        ".long _kernel_vsize",            // SizeOfInitializedData
        ".long 0",                        // SizeOfUninitializedData
        ".long {entry} - _head",     // AddressOfEntryPoint
        ".long 4f - _head",               // BaseOfCode

        // Extra header fields
        ".quad 0",                        // ImageBase
        ".long 0x10000",                  // SectionAlignment (PECOFF_SEGMENT_ALIGN)
        ".long 0x200",                    // FileAlignment (PECOFF_FILE_ALIGN)
        ".short 0",                       // MajorOperatingSystemVersion
        ".short 0",                       // MinorOperatingSystemVersion
        ".short 0",                       // MajorImageVersion
        ".short 0",                       // MinorImageVersion
        ".short 0",                       // MajorSubsystemVersion
        ".short 0",                       // MinorSubsystemVersion
        ".long 0",                        // Win32VersionValue
        ".long _end - _head",             // SizeOfImage
        ".long 4f - _head",               // SizeOfHeaders
        ".long 0",                        // CheckSum
        ".short 10",                      // IMAGE_SUBSYSTEM_EFI_APPLICATION
        ".short 0",                       // DllCharacteristics
        ".quad 0",                        // SizeOfStackReserve
        ".quad 0",                        // SizeOfStackCommit
        ".quad 0",                        // SizeOfHeapReserve
        ".quad 0",                        // SizeOfHeapCommit
        ".long 0",                        // LoaderFlags
        ".long 6",                        // NumberOfRvaAndSizes

        // Data directories
        ".quad 0",                        // ExportTable
        ".quad 0",                        // ImportTable
        ".quad 0",                        // ResourceTable
        ".quad 0",                        // ExceptionTable
        ".quad 0",                        // CertificationTable
        ".quad 0",                        // BaseRelocationTable

        // Section table
        "3:",  // section_table
        // .text section
        ".ascii \".text\\0\\0\\0\"",
        ".long _etext - 4f",              // VirtualSize
        ".long 4f - _head",               // VirtualAddress
        ".long _etext - 4f",              // SizeOfRawData
        ".long 4f - _head",               // PointerToRawData
        ".long 0",                        // PointerToRelocations
        ".long 0",                        // PointerToLineNumbers
        ".short 0",                       // NumberOfRelocations
        ".short 0",                       // NumberOfLineNumbers
        ".long 0x60000020",               // Characteristics

        // .data section
        ".ascii \".data\\0\\0\\0\"",
        ".long _kernel_vsize",            // VirtualSize
        ".long _sdata - _head",           // VirtualAddress
        ".long _kernel_rsize",            // SizeOfRawData
        ".long _sdata - _head",           // PointerToRawData
        ".long 0",                        // PointerToRelocations
        ".long 0",                        // PointerToLineNumbers
        ".short 0",                       // NumberOfRelocations
        ".short 0",                       // NumberOfLineNumbers
        ".long 0xc0000040",               // Characteristics

        ".balign 0x10000",                // PECOFF_SEGMENT_ALIGN
        "4:",  // efi_header_end

        // Jump to kernel_entry
        "b kernel_entry",
        dos_signature = const IMAGE_DOS_SIGNATURE,
        linux_pe_magic = const LINUX_PE_MAGIC,
        phys_link_kaddr = const to_phys(VMLINUX_LOAD_ADDRESS),
        image_nt_signature = const IMAGE_NT_SIGNATURE,
        file_machine = const IMAGE_FILE_MACHINE_LOONGARCH64,
        entry = sym kernel_entry,
    )
}


/// LoongArch64 kernel entry point implementing functionality similar to
/// Linux arch/loongarch/kernel/head.S kernel_entry
#[unsafe(naked)]
#[unsafe(link_section = ".text")]
unsafe extern "C" fn kernel_entry() -> ! {
    naked_asm!(
        // Config direct window and set PG (SETUP_DMWINS)
        "li.d $t0, 0x9000000000000011",   // DMW0: 0x9000..., PLV0, 0x11
        "csrwr $t0, 0x180",               // LOONGARCH_CSR_DMWIN0
        "li.d $t0, 0x8000000000000001",   // DMW1: 0x8000..., PLV0, 0x01
        "csrwr $t0, 0x181",               // LOONGARCH_CSR_DMWIN1

        // Jump to virtual address (JUMP_VIRT_ADDR)
        "pcaddi $t1, 0",                  // Get current PC
        "li.d $t0, 0x9000000000000000",   // Virtual address base
        "or $t0, $t0, $t1",               // Combine with current offset
        "jirl $zero, $t0, 0xc",           // Jump to virtual address

        // Enable PG
        "li.w $t0, 0xb0",                 // PLV=0, IE=0, PG=1
        "csrwr $t0, 0x0",                 // LOONGARCH_CSR_CRMD
        "li.w $t0, 0x04",                 // PLV=0, PIE=1, PWE=0
        "csrwr $t0, 0x1",                 // LOONGARCH_CSR_PRMD
        "li.w $t0, 0x00",                 // FPE=0, SXE=0, ASXE=0, BTE=0
        "csrwr $t0, 0x2",                 // LOONGARCH_CSR_EUEN

        // Clear .bss section
        "la.pcrel $t0, __bss_start",
        "st.d $zero, $t0, 0",
        "la.pcrel $t1, __bss_stop",
        "addi.d $t1, $t1, -8",            // __bss_stop - LONGSIZE
        "1:",
        "addi.d $t0, $t0, 8",             // LONGSIZE = 8
        "st.d $zero, $t0, 0",
        "bne $t0, $t1, 1b",

        // Save firmware arguments (a0, a1, a2)
        "la.pcrel $t0, {fw_arg0}",
        "st.d $a0, $t0, 0",
        "la.pcrel $t0, {fw_arg1}",
        "st.d $a1, $t0, 0",
        "la.pcrel $t0, {fw_arg2}",
        "st.d $a2, $t0, 0",

        // KSave3 used for percpu base, initialized as 0
        "csrwr $zero, 0x33",              // PERCPU_BASE_KS
        // GPR21 used for percpu base (runtime), initialized as 0
        "move $u0, $zero",

        // Set up stack pointer
        "la.pcrel $tp, {init_thread_union}",
        "li.d $sp, {thread_size}",
        "addi.d $sp, $sp, -{pt_size}",    // _THREAD_SIZE - PT_SIZE
        "add.d $sp, $sp, $tp",

        // Jump to Rust entry point
        "b {start_kernel}",

        fw_arg0 = sym FW_ARG0,
        fw_arg1 = sym FW_ARG1,
        fw_arg2 = sym FW_ARG2,
        init_thread_union = sym INIT_THREAD_UNION,
        thread_size = const 0x4000,        // _THREAD_SIZE
        pt_size = const 0x2a0,             // PT_SIZE
        start_kernel = sym rust_start_kernel,
    )
}

// Static storage for firmware arguments
#[unsafe(link_section = ".data")]
static mut FW_ARG0: usize = 0;

#[unsafe(link_section = ".data")]
static mut FW_ARG1: usize = 0;

#[unsafe(link_section = ".data")]
static mut FW_ARG2: usize = 0;

// Initial thread union for stack
#[unsafe(link_section = ".bss")]
static mut INIT_THREAD_UNION: [u8; 0x4000] = [0; 0x4000];

/// Rust kernel start function that will be called from assembly
#[start_code]
fn rust_start_kernel() -> ! {
    use pie_boot_if::BootInfo;

    // Create a basic BootInfo structure
    let mut boot_info = BootInfo::new();

    // Set basic values (these will be properly initialized later)
    boot_info.cpu_id = 0;
    boot_info.kimage_start_lma = core::ptr::null_mut();
    boot_info.kimage_start_vma = core::ptr::null_mut();
    boot_info.fdt = None;
    boot_info.pg_start = core::ptr::null_mut();
    boot_info.debug_console = None;
    boot_info.free_memory_start = core::ptr::null_mut();

    // For now, use an empty memory regions
    static mut EMPTY_REGIONS: [pie_boot_if::MemoryRegion; 0] = [];
    boot_info.memory_regions = unsafe { (&mut EMPTY_REGIONS[..]).into() };

    // Jump to the common entry point
    crate::common::entry::virt_entry(&boot_info);

    // Should never reach here
    loop {
        unsafe {
            core::arch::asm!("idle.0");
        }
    }
}
