extern "C" {
    static HEAP_START: usize;
    static HEAP_SIZE: usize;
}

static mut ALLOC_START: usize = 0;
/// page size per riscv Sv39 spec is 4096 bytes
/// which needs 12 bits to address each byte inside
const PAGE_ADDR_MAGNITIDE: usize = 12;
const PAGE_SIZE: usize = 1 << PAGE_ADDR_MAGNITIDE;
/// a mask with all used bits set
const PAGE_ADDR_MASK: usize = PAGE_SIZE - 1;

/// Produces a page-aligned address by adding one
/// less than the page size (4095), then masking low bits
/// to decrease the address back to the nearest page boundary
const fn align_address(address: usize) -> usize {
    (address + PAGE_ADDR_MASK) & !PAGE_ADDR_MASK
}

#[repr(u8)]
#[derive(PartialEq, Eq, Copy, Clone)]
enum PageFlagBits {
    Empty = 0b00,
    Taken = 0b01,
    Last = 0b10,
    LastTaken = 0b11,
}

#[derive(PartialEq, Eq, Copy, Clone)]
struct PageFlags(u8);

impl PageFlags {
    fn taken(&self) -> bool {
        let test = PageFlagBits::Taken as u8;
        self.0 & test == test
    }
    fn last(&self) -> bool {
        let test = PageFlagBits::Last as u8;
        self.0 & test == test
    }
    fn free(&self) -> bool {
        !self.taken()
    }
}

pub struct Page {
    flags: PageFlags,
}

impl Page {
    pub fn free(&self) -> bool {
        self.flags.free()
    }
    pub fn taken(&self) -> bool {
        self.flags.taken()
    }
}

trait PageTable {}

trait PageEntry {}

// must be aligned to 4096 byte boundary
struct Sv32Table([Sv32Entry; 1024]);

impl Sv32Table {
    pub fn translate(&self, address: Sv32Address) -> usize {
        // double check that we are the recognized page table
        let a = crate::cpu_status::get_satp().ppn() << 12;
        assert!(a as *const Sv32Table == self as *const Sv32Table);
        let mut i = 1;
        let pte = a + (address.vpn()[i] << 12);
        let entry = &self.0[address.vpn()[i]];
        assert!(pte as *const Sv32Entry == entry as *const Sv32Entry);
        assert!(entry.valid() && !(!entry.readable() && entry.writable()));
        if entry.readable() || entry.executable() {
            i = i - 1;
            // todo: descend to next level
            let pte = (a + (address.vpn()[i] << 12)) as *const Sv32Entry;
            let entry = unsafe { pte.as_ref().unwrap() };
            if !entry.valid() || entry.leaf() {
                panic!("unable to translate address");
            } else {
                address.offset() + entry.page_number() << 10
            }
        } else {
            // let ppn = entry.ppn();
            address.offset() + entry.page_number() << 10
        }
    }
}

// if not valid, all other values shall be ignored by hardware. software can freely use.
struct Sv32Entry(u32);

impl Sv32Entry {
    pub fn valid(&self) -> bool {
        self.0 & 0b1 == 1
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
        let value = (value as u32) << 8;
        self.0 = self.0 & ((mask as u32) << 8) | value;
    }
    pub fn page_number(&self) -> usize {
        (self.0 >> 10) as usize
    }
    pub fn ppn(&self) -> [usize; 2] {
        let page = self.page_number();
        [page & ((1 << 10) - 1), page >> 10]
    }
}

struct Sv32Address(u32);

impl Sv32Address {
    pub fn page_number(&self) -> usize {
        self.0 as usize >> 12
    }
    pub fn vpn(&self) -> [usize; 2] {
        let page = self.page_number();
        [page & ((1 << 10) - 1), page >> 10]
    }
    pub fn offset(&self) -> usize {
        (self.0 & (1 << 12) - 1) as usize
    }
}

struct Sv39Table {}

impl PageTable for Sv39Table {
    //
}

pub fn alloc(count: usize) -> *mut u8 {
    assert!(count > 0);
    let total_page_count = unsafe { HEAP_SIZE } / PAGE_SIZE;
    let current = unsafe { HEAP_START } as *mut Page;
    for page in 0..total_page_count - count {
        let mut found = false;
        if unsafe { current.add(page).as_ref() }.unwrap().free() {
            found = true;
