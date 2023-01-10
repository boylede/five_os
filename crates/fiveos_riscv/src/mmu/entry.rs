/// testing out a trait-based interface to the page table entries
pub trait PTEntryRead {
    /// copy the flags out of the entry for inspection
    fn extract_flags(&self) -> EntryFlags;
    /// returns a 32-bit address, if appropriate for table type
    fn address_limited(&self) -> Option<u32>;
    /// returns 32-bit address segment
    fn extract_segment_limited(&self, level: usize) -> u32;
    /// returns the address in 64 bits
    fn address(&self) -> u64;
    fn extract_extended_flags(&self) -> ExtendedFlags;
    fn extract_segment(&self, level: usize) -> u64;
}

/// testing out a trait-based interface to the page table entries
pub trait PTEntryWrite {
    /// overwrite the flags with the ones provided
    fn write_flags(&mut self, flags: EntryFlags);
    fn invalidate(&mut self);
    fn write_address(&mut self, address: u64);
    /// returns the address in 64 bits
    fn write_extended_flags(&mut self) -> ExtendedFlags;
}

/// unimplemented boilerplate for the top 10 bits in larger page table entries
#[derive(Clone, Copy)]
pub struct ExtendedFlags {
    inner: u16,
}

/// low 10 bits in all currently specified page table entry types
#[derive(Clone, Copy)]
pub struct EntryFlags {
    inner: u16,
}

impl EntryFlags {
    #[inline]
    pub const fn new() -> EntryFlags {
        EntryFlags { inner: 0 }
    }
    #[inline]
    pub const fn from_u16(inner: u16) -> EntryFlags {
        EntryFlags { inner }
    }
    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.inner & (1 << 0) != 0
    }
    /// Checks if RWE bits are set wrongly
    #[inline]
    pub const fn is_invalid(&self) -> bool {
        !self.is_readable() && (self.is_writable() || self.is_executable())
    }
    #[inline]
    pub const fn is_readable(&self) -> bool {
        self.inner & (1 << 1) != 0
    }
    #[inline]
    pub const fn is_writable(&self) -> bool {
        self.inner & (1 << 2) != 0
    }
    #[inline]
    pub const fn is_executable(&self) -> bool {
        self.inner & (1 << 3) != 0
    }
    #[inline]
    pub const fn is_user(&self) -> bool {
        self.inner & (1 << 4) != 0
    }
    #[inline]
    pub const fn is_global(&self) -> bool {
        self.inner & (1 << 5) != 0
    }
    #[inline]
    pub const fn is_accessed(&self) -> bool {
        self.inner & (1 << 6) != 0
    }
    #[inline]
    pub const fn is_dirty(&self) -> bool {
        self.inner & (1 << 7) != 0
    }
    #[inline]
    pub const fn read_softflags(&self) -> (bool, bool) {
        (self.inner & (1 << 8) != 0, self.inner & (1 << 9) != 0)
    }
    #[inline]
    pub const fn is_branch(&self) -> bool {
        self.is_valid() && !self.is_readable() && !self.is_writable() && !self.is_executable()
    }

    #[inline]
    pub const fn set_valid(&mut self, flag: bool) {
        let flag = if flag { 1 } else { 0 };
        self.inner |= flag << 0;
    }
    #[inline]
    pub const fn set_readable(&mut self, flag: bool) {
        let flag = if flag { 1 } else { 0 };
        self.inner |= flag << 1;
    }
    #[inline]
    pub const fn set_writable(&mut self, flag: bool) {
        let flag = if flag { 1 } else { 0 };
        self.inner |= flag << 2;
    }
    #[inline]
    pub const fn set_executable(&mut self, flag: bool) {
        let flag = if flag { 1 } else { 0 };
        self.inner |= flag << 3;
    }
    #[inline]
    pub const fn set_user(&mut self, flag: bool) {
        let flag = if flag { 1 } else { 0 };
        self.inner |= flag << 4;
    }
    #[inline]
    pub const fn set_global(&mut self, flag: bool) {
        let flag = if flag { 1 } else { 0 };
        self.inner |= flag << 5;
    }
    #[inline]
    pub const fn set_accessed(&mut self, flag: bool) {
        let flag = if flag { 1 } else { 0 };
        self.inner |= flag << 6;
    }
    #[inline]
    pub const fn set_dirty(&mut self, flag: bool) {
        let flag = if flag { 1 } else { 0 };
        self.inner |= flag << 7;
    }
    #[inline]
    pub const fn set_softflags(&mut self, flag: (bool, bool)) {
        let flag = match flag {
            (true, true) => 0b11,
            (true, false) => 0b10,
            (false, true) => 0b01,
            (false, false) => 0b000,
        };
        self.inner |= flag << 8;
    }
    /// clears the RWE bits, leaving valid bit alone
    #[inline]
    pub const fn set_branch(&mut self) {
        self.inner = self.inner & !0xE;
    }
    #[inline]
    pub const fn with_valid(mut self, flag: bool) -> EntryFlags {
        self.set_valid(flag);
        self
    }
    #[inline]
    pub const fn with_readable(mut self, flag: bool) -> EntryFlags {
        self.set_readable(flag);
        self
    }
    #[inline]
    pub const fn with_writable(mut self, flag: bool) -> EntryFlags {
        self.set_writable(flag);
        self
    }
    #[inline]
    pub const fn with_executable(mut self, flag: bool) -> EntryFlags {
        self.set_executable(flag);
        self
    }
    #[inline]
    pub const fn with_user(mut self, flag: bool) -> EntryFlags {
        self.set_user(flag);
        self
    }
    #[inline]
    pub const fn with_global(mut self, flag: bool) -> EntryFlags {
        self.set_global(flag);
        self
    }
    #[inline]
    pub const fn with_accessed(mut self, flag: bool) -> EntryFlags {
        self.set_accessed(flag);
        self
    }
    #[inline]
    pub const fn with_dirty(mut self, flag: bool) -> EntryFlags {
        self.set_dirty(flag);
        self
    }
    #[inline]
    pub const fn with_softflags(mut self, flag: (bool, bool)) -> EntryFlags {
        self.set_softflags(flag);
        self
    }
    #[inline]
    pub const fn as_u16(self) -> u16 {
        self.inner
    }
    pub const READ: EntryFlags = EntryFlags::new().with_valid(true).with_readable(true);
    pub const READ_WRITE: EntryFlags = EntryFlags::READ.with_writable(true);
    pub const USER_READ_WRITE: EntryFlags = EntryFlags::READ_WRITE.with_user(true);
    pub const READ_EXECUTE: EntryFlags = EntryFlags::READ.with_executable(true);
}
