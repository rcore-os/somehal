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