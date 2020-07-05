extern "C" {
    fn asm_get_misa() -> u64;
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
