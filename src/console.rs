// use crate::uart::Uart;

// #[macro_use]
// use crate::{print, println};

// mod commands;

#[macro_export]
macro_rules! print {
    ($($args:tt)+) => ({
        use core::fmt::Write;
        let _ = write!($crate::cpu::uart::Uart::default(), $($args)+);
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

/*

struct CommandBuffer {
    buffer: [u8; 256],
    cursor: usize,
}
impl CommandBuffer {
    pub fn new() -> Self {
        CommandBuffer {
            buffer: [0; 256],
            cursor: 0,
        }
    }
    pub fn clear(&mut self) {
        assert!(self.cursor < self.buffer.len());
        for i in 0..self.cursor {
            self.buffer[i] = 0;
        }
        self.cursor = 0;
    }
    pub fn run(&mut self) {
        commands::run_command(self.fetch_command());
        self.clear();
    }
    pub fn push(&mut self, value: u8) {
        self.buffer[self.cursor] = value;
        self.cursor += 1;
        if self.cursor == usize::MAX {
            self.clear();
        }
    }
    pub fn set(&mut self, value: u8) {
        self.buffer[self.cursor] = value;
    }
    pub fn fetch_command(&self) -> [u8; 12] {
        unimplemented!()
    }
}

pub struct Console {
    inner: Uart,
    buffer: CommandBuffer,
}

impl Console {
    pub fn new() -> Console {
        Console {
            inner: Uart::default(),
            buffer: CommandBuffer::new(),
        }
    }
    pub fn run(&mut self) {
        self.inner.init();
        loop {
            if let Some(c) = self.inner.get() {
                match c {
                    8 => {
                        print!("{} {}", 8 as char, 8 as char);
                        self.buffer.set(b' ');
                    }
                    0x1b => {
                        println!("esc");
                        break;
                    }
                    0x0a => {
                        println!();
                        self.buffer.clear();
                    }
                    _ => {
                        print!("{}", c as char);
                        self.buffer.push(c);
                    }
                }
            }
        }
        println!("Closing console...");
    }
}

*/
