use super::raw::asm_set_mtvec;

pub fn set_trap_vector(address: usize) {
    // todo: perform sanity checks
    unsafe { asm_set_mtvec(address) };
}
