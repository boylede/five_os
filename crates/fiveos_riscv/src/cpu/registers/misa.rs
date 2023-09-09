use core::arch::asm;

use super::raw::asm_read_misa_xlen;


#[derive(Debug, Clone, Copy)]
pub struct Misa {
    xlen: u8,
    extensions: u32,
}

impl Misa {
    pub const EXTENSION_NAMES: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    pub const EXTENSION_DESCRIPTIONS: [&str; 26] = [
        "Atomics (A)",
        "Reserved (B)",
        "Compressed (C)",
        "Double-precision floating point (D)",
        "Embedded base ISA (E)",
        "Single-precision floating point (F)",
        "Additional standards present (G)",
        "Hypervisor (H)",
        "Integer base ISA (I)",
        "Reserved (J)",
        "Reserved (K)",
        "Reserved (L)",
        "Integer multiply & divide (M)",
        "User-level interrupts (N)",
        "Reserved (O)",
        "Reserved (P)",
        "Quad precision floating point (Q)",
        "Reserved (R)",
        "Supervisor Mode (S)",
        "Reserved (T)",
        "User Mode (U)",
        "Reserved (V)",
        "Reserved (W)",
        "Nonstandard extensions present (X)",
        "Reserved (Y)",
        "Reserved (Z)",
    ];
    const EXTENSION_MASK: usize = (1 << 26) - 1;
    pub fn get() -> Option<Misa> {
        let misa = unsafe {
            let misa: usize;
            asm!("csrr   {}, misa", out(reg) misa);
            misa
        };
        if misa == 0 {
            None
        } else {
            let xlen = unsafe {
                //     let xlen: usize;
                //     asm!("
                //     bltz {misa}, 1f
                //     li {xlen}, 32
                //     ret

                // 1:

                //     srli a0, a0, 1
                //     bltz a0, 2f
                //     li a0, 64
                //     ret
                // 2:
                //     li a0, 128
                //     ret", misa=in(reg) misa, xlen=out(reg) xlen);
                asm_read_misa_xlen()
                // xlen
            } as u8;
            let extensions = (misa & Self::EXTENSION_MASK) as u32;

            Some(Misa { xlen, extensions })
        }
    }
    pub fn xlen(&self) -> u8 {
        self.xlen
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
