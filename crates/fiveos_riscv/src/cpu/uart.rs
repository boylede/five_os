/// Uart driver for riscv virtio spec
use core::fmt::{Error, Write};

pub struct Uart<const B: usize> {
    // base_address: *mut u8,
}

impl<const B: usize> Uart<B> {
    pub fn new() -> Self {
        // let base_address = base_address as *mut u8;
        Uart {}
    }
    pub fn put(&mut self, c: u8) {
        unsafe {
            (B as *mut u8).add(0).write_volatile(c);
        }
    }
    pub fn get(&mut self) -> Option<u8> {
        unsafe {
            // check line control register to see if we have data
            if (B as *mut u8).add(5).read_volatile() & 1 == 0 {
                None
            } else {
                Some((B as *mut u8).add(0).read_volatile())
            }
        }
    }
    pub fn init(&mut self) {
        unsafe {
            // set line control register (3) word length to 8 bits
            (B as *mut u8).add(3).write_volatile(0b11);
            // set fifo FCR enable
            (B as *mut u8).add(2).write_volatile(1);
            // set received data available interrupt
            (B as *mut u8).add(1).write_volatile(1);
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

impl<const B: usize> Default for Uart<B> {
    fn default() -> Uart<B> {
        Uart {
            // base_address: BASE_ADDRESS as *mut u8,
        }
    }
}

impl<const B: usize> Write for Uart<B> {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for c in s.bytes() {
            self.put(c);
        }
        Ok(())
    }
}
