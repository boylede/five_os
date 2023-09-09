use core::arch::asm;

use crate::mmu::align_address_to_page;

extern "C" {
    pub fn asm_get_misa() -> usize;
    pub fn asm_get_mtvec() -> usize;
    pub fn asm_set_mtvec(_: usize);
    pub fn asm_get_satp() -> usize;
    pub fn asm_set_satp(_: usize);
    pub fn asm_get_mvendorid() -> usize;
    pub fn asm_get_marchid() -> usize;
    pub fn asm_get_mimpid() -> usize;
    pub fn asm_get_mhartid() -> usize;
    pub fn asm_get_mstatus() -> usize;
    pub fn asm_set_mstatus(_: usize);
    pub fn asm_get_mepc() -> usize;
    pub fn asm_read_misa_xlen() -> usize;
}

#[derive(Debug, Clone, Copy)]
pub struct Misa {
    xlen: u8,
    extensions: u32,
}

impl Misa {
    pub const EXTENSION_NAMES: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    pub const EXTENSION_DESCRIPTIONS: [&str; 26] = [
        "Atomics",
        "B-reserved",
        "Compressed",
        "Double-precision floating point",
        "Embedded base ISA",
        "Single-precision floating point",
        "G-additional standards present",
        "Hypervisor",
        "Integer base isa",
        "J-reserved",
        "K-reserved",
        "L-reserved",
        "Multiply/divide integers",
        "N-user-level interrupts",
        "O-reserved",
        "P-reserved",
        "Quad precision floating point",
        "R-reserved",
        "Supervisor Mode",
        "T-reserved",
        "User Mode",
        "V-reserved",
        "W-reserved",
        "X-nonstandard extensions present",
        "Y-reserved",
        "Z-reserved",
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

pub fn set_trap_vector(address: usize) {
    // todo: perform sanity checks
    unsafe { asm_set_mtvec(address) };
}

#[derive(Clone, Copy, Debug)]
pub struct Satp(usize);

impl Satp {
    pub fn from_address(address: usize) -> Self {
        let address = align_address_to_page(address);
        let satp = address >> 12;
        Satp(satp)
    }
    pub fn from(address: usize, mode: u8) -> Self {
        let mut base = Self::from_address(address);
        base.set_mode(mode);
        base
    }
    pub fn address(&self) -> usize {
        (self.0 & ((1 << 21) - 1)) << 12
    }
    pub fn mode(&self) -> u8 {
        unimplemented!()
    }
    pub fn asid(&self) -> u16 {
        unimplemented!()
    }
    pub fn ppn(&self) -> usize {
        match Misa::get().map(|misa| misa.xlen()) {
            Some(32) => self.0 & ((1 << 22) - 1),
            Some(64) => self.0 & ((1 << 44) - 1),
            _ => unimplemented!(),
        }
    }
    pub fn set_mode(&mut self, value: u8) {
        match Misa::get().map(|misa| misa.xlen()) {
            Some(32) => unimplemented!(),
            Some(64) => {
                let mut mode: usize = (value & ((1 << 4) - 1)) as usize;
                mode <<= 60;
                self.0 &= (1 << 60) - 1;
                self.0 |= mode;
            }
            _ => unimplemented!(),
        }
    }
    pub fn set_asid(&mut self, _value: u16) {
        unimplemented!()
    }
    pub fn raw(self) -> usize {
        self.0
    }
    pub fn from_raw(raw: usize) -> Satp {
        Satp(raw)
    }
    pub fn get_satp() -> Satp {
        let satp = unsafe { asm_get_satp() };
        Satp(satp)
    }
    pub fn set_satp(satp: &Satp) {
        unsafe { asm_set_satp(satp.raw()) }
    }
}
