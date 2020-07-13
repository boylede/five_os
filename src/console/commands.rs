#[macro_use]
use crate::{print, println};

const COMMANDS: [Command; 256] = initialize_commands();

pub fn run_command(name: [u8; 12]) {
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
    let mut buf = [0; 16];
    let len = (name.len() + 15) & (16 - 1);

    // for i in 0..12 {
    //     buf[i] = name[i];
    // }
    [0; 16]
}

fn stub() {
    println!("Stub Console Command");
}
