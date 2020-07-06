extern "C" {
    fn asm_get_misa() -> usize;
    static asm_trap_vector: usize;
    fn asm_get_mtvec() -> usize;
    fn asm_get_satp() -> usize;
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
    println!("entering asm again");
    let misa = unsafe { asm_get_misa() };

    // todo: bug here? gives csr_mtvec error if removed.
    println!("back from asm");

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

pub fn inspect_trap_vector() {
    println!("trap vector: {:x}", unsafe { asm_trap_vector });
    match unsafe { asm_trap_vector } & 0b11 {
        0b00 => println!("trap vector set correctly"),
        0b01 => println!("trap vector set incorrectly, contains vectored MODE"),
        0b10 => println!("trap vector contains reserved value 2"),
        0b11 => println!("trap vector contains reserved value 3"),
        _ => unreachable!(),
    };
    let mtv = unsafe { asm_get_mtvec() };
    if mtv != unsafe { asm_trap_vector } {
        print!("mtvec has unexpected value: ");
    }
    match unsafe { asm_trap_vector } & 0b11 {
        0b00 => println!("Direct Mode"),
        0b01 => println!("Vectored Mode"),
        0b10 => println!("Reserved Value 2 Set"),
        0b11 => println!("Reserved Value 3 Set"),
        _ => unreachable!(),
    };
    println!("mtvec: {:x}", mtv);
}

pub struct Satp(usize);

impl Satp {
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
}

pub fn get_satp() -> Satp {
    let satp = unsafe { asm_get_satp() };
    Satp(satp)
}
