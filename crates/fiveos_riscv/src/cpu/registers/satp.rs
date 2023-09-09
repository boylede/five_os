use core::arch::asm;

use crate::mmu::align_address_to_page;

use super::{
    misa::Misa,
    raw::{asm_get_satp, asm_set_satp},
};

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
