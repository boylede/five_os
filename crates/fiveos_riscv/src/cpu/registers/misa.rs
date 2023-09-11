use core::arch::asm;

use num_enum::{FromPrimitive, IntoPrimitive};

const EXTENSION_MASK: usize = (1 << 26) - 1;
const XLEN: usize = core::mem::size_of::<usize>() * 8;
const BASE: usize = 0b11 << (XLEN - 2);

#[derive(Debug, Clone, Copy, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Architecture {
    #[num_enum(default)]
    Unknown = 0,
    ThirtyTwo = 1,
    SixtyFour = 2,
    OneTwentyEight = 3,
}

/// The Machine Instruction Set Architecture register
#[derive(Debug, Clone, Copy)]
pub struct Misa {
    base: Architecture,
    extensions: u32,
}

impl Misa {
    pub const EXPECTED_XLEN: usize = XLEN;
    pub const EXTENSION_NAMES: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    pub const EXTENSION_DESCRIPTIONS: [&'static str; 26] = [
        "Atomics (A)",
        "Reserved (B)", // Bit-manipulation
        "Compressed (C)",
        "Double-precision floating point (D)",
        "Embedded base ISA (E)",
        "Single-precision floating point (F)",
        "Additional standards present (G)",
        "Hypervisor (H)",
        "Integer base ISA (I)",
        "Reserved (J)", // dynamically translated languages
        "Reserved (K)",
        "Reserved (L)",
        "Integer multiply & divide (M)",
        "User-level interrupts (N)",
        "Reserved (O)",
        "Reserved (P)", // packed-simd
        "Quad precision floating point (Q)",
        "Reserved (R)",
        "Supervisor Mode (S)",
        "Reserved (T)",
        "User Mode (U)",
        "Reserved (V)", // vector
        "Reserved (W)",
        "Nonstandard extensions present (X)",
        "Reserved (Y)",
        "Reserved (Z)",
    ];

    pub fn get() -> Option<Misa> {
        let misa = unsafe {
            let misa: usize;
            asm!("csrr   {}, misa", out(reg) misa);
            misa
        };
        if misa == 0 {
            None
        } else {
            let base: Architecture = (((misa & BASE) >> (XLEN - 2)) as u8).into();
            let extensions = (misa & EXTENSION_MASK) as u32;
            Some(Misa { base, extensions })
        }
    }
    pub fn base(&self) -> Architecture {
        self.base
    }
    pub fn xlen(&self) -> usize {
        use Architecture as A;
        match self.base {
            A::Unknown => unreachable!(),
            A::ThirtyTwo => 32,
            A::SixtyFour => 64,
            A::OneTwentyEight => 128,
        }
    }
    pub fn extensions(&self) -> u32 {
        self.extensions
    }
    pub fn extension_name(extension: u8) -> Option<char> {
        Self::EXTENSION_NAMES.chars().nth(extension as usize)
    }
    pub fn extension_description(extension: u8) -> Option<&'static str> {
        Self::EXTENSION_DESCRIPTIONS
            .into_iter()
            .nth(extension as usize)
    }
}
