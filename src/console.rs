// use core::fmt::Write;

use crate::uart::Uart;

#[macro_export]
macro_rules! print {
    ($($args:tt)+) => ({
        use core::fmt::Write;
        let _ = write!(crate::uart::Uart::default(), $($args)+);
    });
}
#[macro_export]
macro_rules! println
{
	() => ({
		print!("\r\n")
	});
	($fmt:expr) => ({
		print!(concat!($fmt, "\r\n"))
	});
	($fmt:expr, $($args:tt)+) => ({
		print!(concat!($fmt, "\r\n"), $($args)+)
	});
}

pub struct Console {
    inner: Uart,
}

impl Console {
    pub fn new() -> Console {
        Console {
            inner: Uart::default(),
        }
    }
    pub fn run(&mut self) {
        self.inner.init();
        loop {
            if let Some(c) = self.inner.get() {
                match c {
                    8 => {
                        print!("{}{}{}", 8 as char, ' ', 8 as char);
                    }
                    0x1b => {
                        break;
                    }
                    0x0a => {
                        println!();
                    }
                    _ => {
                        print!("{}", c as char);
                    }
                }
            }
        }
    }
}
