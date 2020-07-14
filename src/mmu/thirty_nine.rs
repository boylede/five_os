
use crate::mmu::PAGE_SIZE;

pub struct Sv39Entry(u64);

impl Sv39Entry {
    pub fn valid(&self) -> bool {
        self.0 & 0b1 == 1
    }
    pub fn set_valid(&mut self) {
        self.0 = self.0 | 0b1
    }
    pub fn leaf(&self) -> bool {
        self.0 & 0b1110 != 0
    }
    pub fn readable(&self) -> bool {
        self.0 & 0b10 == 0b10
    }
    pub fn writable(&self) -> bool {
        // writable pages must also be readable
        self.0 & 0b100 == 0b100
    }
    pub fn executable(&self) -> bool {
        self.0 & 0b1000 == 0b1000
    }
    // must not be a leaf. otherwise user shall be set to 0 by software
    pub fn user(&self) -> bool {
        self.0 & (1 << 4) == (1 << 4)
    }
    pub fn global(&self) -> bool {
        self.0 & (1 << 5) == (1 << 5)
    }
    // must not be a leaf. otherwise access shall be set to 0 by software
    pub fn accessed(&self) -> bool {
        self.0 & (1 << 6) == (1 << 6)
    }
    // must not be a leaf. otherwise dirty shall be set to 0 by software
    pub fn dirty(&self) -> bool {
        self.0 & (1 << 7) == (1 << 7)
    }
    pub fn get_software(&self) -> u8 {
        let mask = 0b11 << 8;
        (self.0 & (mask >> 8)) as u8
    }
    pub fn set_software(&mut self, value: u8) {
        let mask = 0b11;
        let value = value & mask;
        let value = (value as u64) << 8;
        self.0 = self.0 & ((mask as u64) << 8) | value;
    }
    pub fn page_number(&self) -> usize {
        (self.0 >> 10) as usize
    }
    pub fn ppn(&self) -> [usize; 3] {
        let page = self.page_number();
        [
            page & ((1 << 9) - 1),
            page & ((1 << 9) - 1) >> 9,
            page & ((1 << 9) - 1) >> 18,
        ]
    }
}

pub struct Sv39Table([Sv39Entry; 512]);

impl Sv39Table {
    pub fn at_address(address: usize) -> *mut Sv39Table {
        let address = address as *mut u8;
        for i in 0..PAGE_SIZE {
            unsafe { *(address.add(i)) = 0 };
        }
        address as *mut Sv39Table
    }
    pub fn alloc(&mut self, count: usize) {
        let mut _found = false;
        for i in 0..512 {
            if !self.0[i].valid() {
                _found = true;
                for _j in i..i + count - 1 {
                    if self.0[i].valid() {
                        _found = false;
                        break;
                    }
                }
            }
        }
        unimplemented!()
    }
}

pub struct Sv39Address(u64);

impl Sv39Address {
    pub fn offset(&self) -> u64 {
        self.0 & (1 << 12) - 1
    }
    pub fn vpn(&self) -> [u64; 3] {
        let page = self.page_number();
        [
            (page & ((1 << 9) - 1)) >> 12,
            (page & ((1 << 9) - 1)) >> 21,
            page >> 30,
        ]
    }
    pub fn page_number(&self) -> u64 {
        self.0 >> 12
    }
}