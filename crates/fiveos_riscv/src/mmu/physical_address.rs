trait PhysicalAddress {
    fn as_u64(&self) -> u64;
    fn as_usize(&self) -> usize;
}
