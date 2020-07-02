/// Uart driver for riscv virtio spec
use core::fmt::{Error, Write};

pub struct Uart {
    base_address: *mut u8,
}

impl Uart {
    pub fn new(base_address: usize) -> Self {
        let base_address = base_address as *mut u8;
        Uart { base_address }
    }
    pub fn put(&mut self, c: u8) {
        unsafe {
            self.base_address.add(0).write_volatile(c);
        }
    }
    pub fn get(&mut self) -> Option<u8> {
        unsafe {
            // check line control register to see if we have data
            if self.base_address.add(5).read_volatile() & 1 == 0 {
                None
            } else {
                Some(self.base_address.add(0).read_volatile())
            }
        }
    }
    pub fn init(&mut self) {
        unsafe {
            // set line control register (3) word length to 8 bits
            self.base_address.add(3).write_volatile(0b11);
            // set fifo FCR enable
            self.base_address.add(2).write_volatile(1);
            // set received data available interrupt
            self.base_address.add(1).write_volatile(1);
            // @TODO: calculate divisor LEAST & MOST (when implementing on real hardware) and set
            // // set latch bit (7) to 1 to allow access to divisor
            // self.base_address.add(3).write_volatile(0b11 | 1 << 7);
            // self.base_address.add(0).write_volatile(LEAST);
            // self.base_address.add(1).write_volatile(MOST);
            // // unset latch
            // self.base_address.add(3).write_volatile(0b11);
        }
    }
}

impl Default for Uart {
    fn default() -> Uart {
        Uart {
            base_address: 0x1000_0000 as *mut u8,
        }
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for c in s.bytes() {
            self.put(c);
        }
        Ok(())
    }
}
