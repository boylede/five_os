//! Uart driver for riscv virtio spec

pub const UART_BASE_ADDRESS: usize = 0x1000_0000;
pub const UART_SIZE: usize = 0x100;
pub const UART_END_ADDRESS: usize = UART_BASE_ADDRESS + UART_SIZE;

use core::{
    fmt::{Error, Write},
    ops::{Deref, DerefMut},
};

pub struct Uart0(pub(crate) Uart<UART_BASE_ADDRESS>);

impl Uart0 {
    pub const unsafe fn new() -> Self {
        Uart0(Uart::new())
    }
}

impl Write for Uart0 {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        <Uart<UART_BASE_ADDRESS> as Write>::write_str(&mut self.0, s)
    }
}

impl Deref for Uart0 {
    type Target = Uart<UART_BASE_ADDRESS>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Uart0 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// ZST for direct Uart access
///
pub struct Uart<const B: usize> {}

impl<const B: usize> Uart<B> {
    /// Safety: Uart is used for direct hardware access.
    /// The user is expected to ensure only one instance is created
    pub const unsafe fn new() -> Self {
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

/// todo: move to a fiveos lib
// pub trait RawConsole {
//     type Error;
//     fn initialize(self) -> Self;
//     fn write_str(&mut self, s: &str) -> Result<usize, Self::Error>;
// }

impl<const B: usize> Write for Uart<B> {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for c in s.bytes() {
            // Safety: instantiation of Uart is limited to unsafe contexts
            // it is expected that other protection is used
            unsafe {
                self.put(c);
            }
        }
        Ok(())
    }
}
