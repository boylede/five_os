#[repr(transparent)]
pub struct PageMarker {
    flags: Pageflags,
}

impl PageMarker {
    pub fn is_free(&self) -> bool {
        self.flags.is_empty()
    }
    pub fn is_taken(&self) -> bool {
        !self.flags.is_empty()
    }
    pub fn is_last(&self) -> bool {
        self.flags.is_last()
    }
    pub fn clear(&mut self) {
        self.flags.clear();
    }
    pub fn set_taken(&mut self) {
        self.flags.set_taken();
    }
    pub fn set_last(&mut self) {
        self.flags.set_last();
    }
}

#[repr(transparent)]
struct Pageflags(u8);

impl Pageflags {
    pub fn is_taken(&self) -> bool {
        self.0 & 0b1 == 0b1
    }
    pub fn is_last(&self) -> bool {
        self.0 & 0b10 == 0b10
    }
    pub fn is_empty(&self) -> bool {
        self.0 & 0b1 == 0b0
    }
    pub fn owner(&self) -> u8 {
        self.0 >> 2
    }
    pub fn set_taken(&mut self) {
        self.0 |= 0b1;
    }
    pub fn set_last(&mut self) {
        self.0 |= 0b10;
    }
    pub fn clear(&mut self) {
        self.0 = 0;
    }
    pub fn set_owner(&mut self, value: u8) {
        let mut mask = value & ((1 << 6) - 1);
        mask <<= 2;
        self.0 = (self.0 & 0b11) | mask;
    }
}
