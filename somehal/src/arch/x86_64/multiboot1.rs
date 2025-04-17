use core::arch::{global_asm, naked_asm};

use x86::msr::IA32_EFER;
use x86_64::registers::control::{Cr0Flags, Cr4Flags};
use x86_64::registers::model_specific::EferFlags;

use crate::consts::*;

