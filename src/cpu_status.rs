extern "C" {
    fn asm_get_misa() -> usize;
    fn asm_get_mtvec() -> usize;
    fn asm_set_mtvec(_: usize);
    fn asm_get_satp() -> usize;
    fn asm_set_satp(_: usize);
    fn asm_get_mvendorid() -> usize;
    fn asm_get_marchid() -> usize;
    fn asm_get_mimpid() -> usize;
    fn asm_get_mhartid() -> usize;
    fn asm_get_mstatus() -> usize;
    fn asm_get_mepc() -> usize;
}

#[macro_use]
use crate::{print, println};
use crate::layout::StaticLayout;

#[derive(Debug)]
pub struct Misa {
    xlen: u8,
    extensions: u32,
}

const EXTENSION_NAMES: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

const EXTENSION_DESCRIPTIONS: [&str; 26] = [
    "Atomic",
    "reserved B",
    "Compressed",
    "Double-precision floating point",
    "rv32E base isa",
    "single-precision Floating point",
    "\"additional standards present (G)\"",
    "Hypervisor",
    "rv32I/64I/128I base isa",
    "reserved J",
    "reserved K",
    "reserved L",
    "integer Multiply/divide",
    "user-level interrupts (N)",
    "reserved O",
    "reserved P",
    "Quad precision floating point",
    "reserved R",
    "Supervisor Mode",
    "reserved T",
    "User Mode",
    "reserved V",
    "reserved W",
    "\"non-standard eXtensions present\"",
    "reserved Y",
    "reserved Z",
];

pub fn print_misa_info() {
    println!("--- MISA INFO ---");
    let misa = unsafe { asm_get_misa() };
    let xlen = {
        let mut misa: i64 = misa as i64;
        // if sign bit is 0, XLEN is 32
        if misa > 0 {
            32
        } else {
            // shift misa over 1 bit to check next-highest bit
            misa = misa << 1;
            // if new sign bit is 0, XLEN is 64
            if misa > 0 {
                64
            } else {
                // both high bits are 1, so xlen is 128
                128
            }
        }
    };
    let checked_width = get_base_width();
    if xlen != checked_width {
        println!(
            "ERROR: MISA reports different base width than empirically found: {} vs {}",
            xlen, checked_width
        );
    } else {
        println!("Base ISA Width: {}", xlen);
    }

    let extensions = misa & 0x01FF_FFFF;
    print!("Extensions: ");
    for (i, letter) in EXTENSION_NAMES.chars().enumerate() {
        let mask = 1 << i;
        if extensions & mask > 0 {
            print!("{}", letter);
        }
    }
    println!();
    println!("--- Extensions ---");
    for (i, desc) in EXTENSION_DESCRIPTIONS.iter().enumerate() {
        let mask = 1 << i;
        if extensions & mask > 0 {
            println!("{}", desc);
        }
    }
}

fn get_base_width() -> u64 {
    let mut test: u64 = 4;
    test = test << 31;
    if test == 0 {
        return 32;
    }
    test = test << 31;
    if test > 0 {
        return 128;
    }
    return 64;
}

fn set_trap_vector(address: usize) {
    // todo: perform sanity checks
    unsafe { asm_set_mtvec(address) };
}

pub fn setup_trap() {
    let address = StaticLayout::get().trap_vector;
    let mask = address & 0b11;
    if mask != 0 {
        panic!("Trap vector not aligned to 4-byte boundary: {:x}", address);
    }
    set_trap_vector(address);
}

pub fn inspect_trap_vector() {
    println!("----------- Trap --------------");
    let mtvec = unsafe { asm_get_mtvec() };
    if mtvec == 0 {
        println!("trap vector not initialized");
        return;
    }
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
    pub fn address(&self) -> usize {
        (self.0 & (1 << 21) - 1) << 12
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
            32 => (unimplemented!()),
            64 => {
                let mut mode: usize = (value & (1 << 4) - 1) as usize;
                mode = mode << 60;
                self.0 = self.0 & (1 << 60) - 1;
                self.0 = self.0 | mode;
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
}

pub fn get_satp() -> Satp {
    let satp = unsafe { asm_get_satp() };
    Satp(satp)
}

pub fn set_satp(satp: &Satp) {
    unsafe { asm_set_satp(satp.raw()) }
}

pub fn print_cpu_info() {
    let vendor = unsafe { asm_get_mvendorid() };
    let architecture = unsafe { asm_get_marchid() };
    let implementation = unsafe { asm_get_mimpid() };
    println!("--- CPU INFO ---");
    println!("Vendor: {:x}", vendor);
    println!("Architecture: {:x}", architecture);
    println!("Implementaton: {:x}", implementation);
}

pub fn print_trap_info() {
    let mepc = unsafe { asm_get_mepc() };
    println!("mepc: {:x}", mepc);
    let mtvec = unsafe { asm_get_mtvec() };
    println!("mtvec: {:x}", mtvec);
}
