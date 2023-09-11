use crate::{print, println};

const COMMANDS: [Command; 256] = initialize_commands();

pub fn run_command(_name: [u8; 12]) {
    unimplemented!()
}

type CommandName = [u8; 16];

#[derive(Copy, Clone)]
struct Command {
    name: [u8; 16],
    function: ConstFn,
}

#[derive(Copy, Clone)]
#[repr(transparent)]
struct ConstFn(*const fn());

const STUBFN: ConstFn = ConstFn(stub as *const fn());

const fn initialize_commands() -> [Command; 256] {
    [Command {
        name: encode_str("quit"),
        function: STUBFN,
    }; 256]
}

const fn encode_str(name: &str) -> [u8; 16] {
    let mut _buf = [0; 16];
    let _len = (name.len() + 15) & (16 - 1);

    // for i in 0..12 {
    //     buf[i] = name[i];
    // }
    [0; 16]
}

fn stub() {
    println!("Stub Console Command");
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
