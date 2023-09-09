pub mod misa;
pub mod mstatus;
pub mod mtvec;
pub mod satp;

pub mod raw {
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
}
