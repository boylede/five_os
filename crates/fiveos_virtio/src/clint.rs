//! VIRTIO Core Local Interruptor

pub const CLINT_BASE_ADDRESS: usize = 0x0200_0000;
pub const CLINT_SIZE: usize = 0x1_0000;
pub const CLINT_END_ADDRESS: usize = CLINT_BASE_ADDRESS + CLINT_SIZE;
