/// todo: move to own lib
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct GreyCode(u32);

impl GreyCode {
    pub fn new() -> GreyCode {
        GreyCode(0)
    }

    pub fn peek_next(&self) -> GreyCode {
        GreyCode(increment_grey(self.0))
    }
    pub fn increment(&mut self) {
        self.0 = increment_grey(self.0)
    }
    pub fn get(&self) -> u32 {
        self.0
    }
}

/// adapted from http://www-graphics.stanford.edu/~seander/bithacks.html
const fn parity_is_even(mut n: u32) -> bool {
    n ^= n >> 1;
    n ^= n >> 2;
    n = (n & 0x11111111) * 0x11111111;
    return (n >> 28) & 0b1 != 1;
}

/// implementation from https://stackoverflow.com/a/17493235
const fn increment_grey(mut n: u32) -> u32 {
    if parity_is_even(n) {
        n ^= 0b1;
        n
    } else {
        let y = n & !(n - 1);
        let y = y << 1;
        n ^= y;
        n
    }
}
