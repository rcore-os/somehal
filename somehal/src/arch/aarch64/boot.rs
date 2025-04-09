use crate::mem::boot::*;

const FLAG_LE: usize = 0b0;
const FLAG_PAGE_SIZE_4K: usize = 0b10;
const FLAG_ANY_MEM: usize = 0b1000;

#[naked]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot.header")]
/// The entry point of the kernel.
pub unsafe extern "C" fn _start() -> ! {
    unsafe {
        naked_asm!(
            // code0/code1
            "nop",
            "bl {entry}",
            // text_offset
            ".quad 0",
            // image_size
            ".quad __kernel_load_size",
            // flags
            ".quad {flags}",
            // Reserved fields
            ".quad 0",
            ".quad 0",
            ".quad 0",
            // magic - yes 0x644d5241 is the same as ASCII string "ARM\x64"
            ".ascii \"ARM\\x64\"",
            // Another reserved field at the end of the header
            ".byte 0, 0, 0, 0",
            flags = const FLAG_LE | FLAG_PAGE_SIZE_4K | FLAG_ANY_MEM,
            entry = sym primary_entry,
        )
    }
}

#[allow(unused)]
#[link_boot::link_boot]
mod _m {
    use core::arch::{asm, naked_asm};

    use aarch64_cpu::{asm::barrier, registers::*};

    use crate::arch::paging::enable_mmu;
    use crate::consts::KERNEL_STACK_SIZE;
    use crate::dbgln;
    use crate::fdt::set_fdt_ptr;

    #[naked]
    /// The entry point of the kernel.
    unsafe extern "C" fn primary_entry() -> ! {
        unsafe {
            naked_asm!(
                "ADR      x11, .",
                "LDR      x10, ={this_func}",
                "SUB      x18, x10, x11", // x18 = va_offset
                "MOV      x19, x0",        // x19 = dtb_addr

                // setup stack
                "LDR      x1,  =__stack_bottom",
                "ADD      x1,  x1, {stack_size}",
                "SUB      x1,  x1, x18", // X1 == STACK_TOP
                "MOV      sp,  x1",

                "BL       {switch_to_elx}",

                "LDR      x0, =vector_table_el1",
                "MSR      VBAR_EL1, x0",

                "MOV      x0,  x18",
                "MOV      x1,  x19",
                "BL       {entry}",
                this_func = sym primary_entry,
                stack_size = const KERNEL_STACK_SIZE,
                switch_to_elx = sym switch_to_elx,
                entry = sym rust_boot,
            )
        }
    }

    fn rust_boot(kcode_va: usize, fdt: *mut u8) -> ! {
        unsafe {
            clean_bss();
            enable_fp();
            set_kcode_va_offset(kcode_va);
            set_fdt_ptr(fdt);

            #[cfg(feature = "early-debug")]
            super::debug::init();
            dbgln!("Booting up");
            dbgln!("Entry      : {}", kernal_load_addr().raw());
            dbgln!("Code offset: {}", kcode_offset());
            dbgln!("Current EL : {}", CurrentEL.read(CurrentEL::EL));
            dbgln!("fdt        : {}", fdt);
            dbgln!("fdt size   : {}", crate::fdt::fdt_size());
        }
        enable_mmu()
    }

    /// Switch to EL1.
    #[cfg(not(feature = "vm"))]
    fn switch_to_elx() {
        SPSel.write(SPSel::SP::ELx);
        SP_EL0.set(0);
        let current_el = CurrentEL.read(CurrentEL::EL);
        if current_el >= 2 {
            if current_el == 3 {
                // Set EL2 to 64bit and enable the HVC instruction.
                SCR_EL3.write(
                    SCR_EL3::NS::NonSecure
                        + SCR_EL3::HCE::HvcEnabled
                        + SCR_EL3::RW::NextELIsAarch64,
                );
                // Set the return address and exception level.
                SPSR_EL3.write(
                    SPSR_EL3::M::EL1h
                        + SPSR_EL3::D::Masked
                        + SPSR_EL3::A::Masked
                        + SPSR_EL3::I::Masked
                        + SPSR_EL3::F::Masked,
                );
                unsafe {
                    asm!(
                    "adr    x2, {}",
                    "msr    elr_el3, x2",
                     sym primary_entry
                        );
                }
            }
            // Disable EL1 timer traps and the timer offset.
            CNTHCTL_EL2.modify(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);
            CNTVOFF_EL2.set(0);
            // Set EL1 to 64bit.
            HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);
            // Set the return address and exception level.
            SPSR_EL2.write(
                SPSR_EL2::M::EL1h
                    + SPSR_EL2::D::Masked
                    + SPSR_EL2::A::Masked
                    + SPSR_EL2::I::Masked
                    + SPSR_EL2::F::Masked,
            );
            unsafe {
                asm!(
                    "
            mov     x8, sp
            msr     sp_el1, x8
            MOV     x0, x19
            adr     x2, {}
            msr     elr_el2, x2
            eret
            " , 
                sym primary_entry
                )
            };
        }
    }

    /// Switch to EL2.
    #[cfg(feature = "vm")]
    fn switch_to_elx() {
        SPSel.write(SPSel::SP::ELx);
        let current_el = CurrentEL.read(CurrentEL::EL);
        if current_el == 3 {
            SCR_EL3.write(
                SCR_EL3::NS::NonSecure + SCR_EL3::HCE::HvcEnabled + SCR_EL3::RW::NextELIsAarch64,
            );
            SPSR_EL3.write(
                SPSR_EL3::M::EL2h
                    + SPSR_EL3::D::Masked
                    + SPSR_EL3::A::Masked
                    + SPSR_EL3::I::Masked
                    + SPSR_EL3::F::Masked,
            );
            ELR_EL3.set(LR.get());
            aarch64_cpu::asm::eret();
        }

        // Set EL1 to 64bit.
        // Enable `IMO` and `FMO` to make sure that:
        // * Physical IRQ interrupts are taken to EL2;
        // * Virtual IRQ interrupts are enabled;
        // * Physical FIQ interrupts are taken to EL2;
        // * Virtual FIQ interrupts are enabled.
        HCR_EL2.modify(
            HCR_EL2::VM::Enable
            + HCR_EL2::RW::EL1IsAarch64
            + HCR_EL2::IMO::EnableVirtualIRQ // Physical IRQ Routing.
            + HCR_EL2::FMO::EnableVirtualFIQ // Physical FIQ Routing.
            + HCR_EL2::TSC::EnableTrapEl1SmcToEl2,
        );
    }
    fn enable_fp() {
        CPACR_EL1.write(CPACR_EL1::FPEN::TrapNothing);
        barrier::isb(barrier::SY);
    }
}
