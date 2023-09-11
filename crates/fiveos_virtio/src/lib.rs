#![no_std]

use core::marker::PhantomData;

use uart::{Uart, Uart0};

pub mod clint;
pub mod plic;
pub mod rtc;
pub mod uart;

pub static mut PERIPHERALS: Option<Peripherals> = Some(Peripherals {
    uart: unsafe { Uart0::new() },
});

pub struct Peripherals {
    pub uart: Uart0,
}

impl Peripherals {
    // fn take_uart0(&mut self) -> Uart0 {
    //     let p = core::mem::replace(&mut self.uart, None);
    //     p.unwrap()
    // }
}
