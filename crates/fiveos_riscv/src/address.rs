#[repr(transparent)]
pub struct PhysicalAddress(u64);
pub struct InvalidPhysicalAddress(u64);
#[repr(transparent)]
pub struct VirtualAddress(u64);
