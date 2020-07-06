extern "C" {
    fn asm_get_misa() -> usize;
    static asm_trap_vector: usize;
    fn asm_get_mtvec() -> usize;
    fn asm_set_mtvec(_: usize);
    fn asm_get_satp() -> usize;
    fn asm_set_satp(_:usize);
}

#[macro_use]
use crate::{print, println};

#[derive(Debug)]
pub struct Misa {
    xlen: u8,
    extensions: u32,
}

const EXTENSION_NAMES: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub fn print_misa_info() {
    let misa = unsafe { asm_get_misa() };

    // todo: bug here? gives csr_mtvec error if removed.

    let xlen = {
        let mut misa: i64 = misa as i64;
        if misa > 0 {
            32
        } else {
            misa << 1;
            if misa > 0 {
                128
            } else {
                64
            }
        }
    };
    let checked_width = get_base_width();
    if xlen != checked_width {
        print!(
            "MISA reports different base width than empirically found: {} vs {}",
            xlen, checked_width
        );
    } else {
        print!("MISA reports base width {}", xlen);
    }

    let extensions = misa & 0x01FF_FFFF;
    print!(" and found extensions ");
    for (i, letter) in EXTENSION_NAMES.chars().enumerate() {
        let mask = 1 << i;
        if extensions & mask > 0 {
            print!("{}", letter);
        }
    }
    println!();
}

fn get_base_width() -> u64 {
    let mut test: u64 = 4;
    test << 31;
    if test == 0 {
        return 32;
    }
    test << 31;
    if test > 0 {
        return 64;
    }
    return 128;
}

fn set_trap_vector(address: usize) {
    // todo: perform sanity checks
    unsafe { asm_set_mtvec(address) };
}

pub fn setup_trap() {
    let mut address = unsafe { asm_trap_vector };
    print!("setting trap: {:x}", address);
    let address_alignment = 4 - 1;
    address += address_alignment;
    address = address & !address_alignment;
    println!(" -> {:x}", address);
    set_trap_vector(address);
}

pub fn inspect_trap_vector() {
    let mtvec = unsafe { asm_get_mtvec() };
    println!("trap vector: {:x}", mtvec);
    match mtvec & 0b11 {
        0b00 => println!("Direct Mode"),
        0b01 => println!("Vectored Mode"),
        0b10 => println!("Reserved Value 2 Set"),
        0b11 => println!("Reserved Value 3 Set"),
        _ => unreachable!(),
    };
}

#[derive(Clone, Copy)]
pub struct Satp(usize);

impl Satp {
    pub fn from_address(address: usize) -> Self {
        let address = crate::page::align_address(address);
        let satp = address >> 12;
        Satp(satp)
    }
    pub fn mode(&self) -> u8 {
        unimplemented!()
    }
    pub fn asid(&self) -> u16 {
        unimplemented!()
    }
    pub fn ppn(&self) -> usize {
        match get_base_width() {
            32 => self.0 & ((1 << 22) - 1),
            64 => self.0 & ((1 << 44) - 1),
            _ => unimplemented!(),
        }
    }
    pub fn set_mode(&mut self, value: u8) {
        match get_base_width() {
            32 => (
                unimplemented!()
            ),
            64 => {
                let mut mode: usize = (value & ( 1 << 4) - 1) as usize;
                mode = mode << 60;
                self.0 = self.0 & (1 << 60) - 1;
                self.0 = self.0 | mode;
            },
            _ => unimplemented!(),
        }
    }
    pub fn set_asid(&mut self, value: u16) {
        unimplemented!()
    }
    pub fn raw(self) -> usize {
        self.0
    }
}

pub fn get_satp() -> Satp {
    let satp = unsafe { asm_get_satp() };
    Satp(satp)
}

pub fn set_satp(satp: &Satp) {
    unsafe {asm_set_satp(satp.raw())}
}
