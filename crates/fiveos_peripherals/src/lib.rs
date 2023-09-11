#![no_std]

// mod commands;
pub mod macros;

use core::{
    cell::UnsafeCell,
    marker::PhantomData,
    mem::MaybeUninit,
    sync::atomic::{AtomicPtr, AtomicU32, AtomicU64, AtomicU8, Ordering, ATOMIC_USIZE_INIT},
};

// use fiveos_virtio::uart::RawConsole;

// pub struct AtomicConsole<T>(T);

// impl<T> AtomicConsole<T>
// where
//     T: RawConsole,
// {
//     pub fn new(hardware: T) -> AtomicConsole<T> {
//         let hardware = hardware.initialize();
//         AtomicConsole(hardware)
//     }
//     pub fn send(&self, item: T) {

//     }
// }

// enum ItemState<T> {
//     Free,
//     Reserved,
//     Committed(*const T),
//     Poisoned,
// }

// pub struct AtomicCollection<T> {
//     data: [AtomicU64; 64],
//     _marker: PhantomData<T>,
// }

// impl<T> AtomicCollection<T> {
//     pub fn new() -> AtomicCollection<T> {
//         const ATOMIC_ZERO: AtomicU64 = AtomicU64::new(0);
//         let data = [ATOMIC_ZERO; 64];
//         AtomicCollection {
//             data: data,
//             _marker: PhantomData,
//         }
//     }
//     fn insert(&self, item: &T) -> Option<u8> {
//         let pointer = item as *const T as u64 | 0b10 << 62;
//         for value in self.data.iter() {
//             let entry = value.load(Ordering::Relaxed);
//             let state = (( & (0b11 << 62)) >> 62) as u8;
//             const LO_MASK: u64 = (1 << 32) - 1;
//             let grey = GreyCode((entry & LO_MASK) as u32).peek_next();
//             match state {
//                 // Free
//                 0b00 => {
//                     value.compare_exchange(value, new, success, failure)
//                     value.store(pointer, Ordering::Relaxed);
//                 },
//                 // Reserved
//                 0b01 => (),
//                 // Committed
//                 0b10 => (),
//                 // Poisoned
//                 0b11 => (),
//                 _ => unreachable!(),
//             }
//         }
//         None

//     }
// }

// impl<T> AtomicQueue<T> {
//     pub fn new() -> AtomicQueue<T> {
//         // Safety: arrays of MaybeUninit do not need to be initialized
//         let data: [MaybeUninit<T>; 256] = unsafe { MaybeUninit::uninit().assume_init() };
//         AtomicQueue {
//             cursor: AtomicCursor::new(),
//             data: UnsafeCell::new(data),
//         }
//     }
//     pub fn send(&self, item: T) {
//         let _ = item;
//         todo!()
//     }
//     pub fn receive(&self) -> Option<T> {
//         todo!()
//     }
//     pub fn len(&self) -> usize {
//         let read = 0;
//         todo!()
//     }
// }

// pub struct AtomicCursor {
//     inner: AtomicU64,
// }

// impl AtomicCursor {
//     pub fn new() -> AtomicCursor {
//         AtomicCursor {
//             inner: AtomicU64::new(0),
//         }
//     }
//     fn read(&self) -> CursorSnapshot {
//         CursorSnapshot::from(self)
//     }
//     pub fn read_reserve(&self) -> Result<u8, ()> {
//         let mut snapshot = self.read();
//         snapshot.increment_reader();
//         if snapshot.try_commit() {
//             Ok(snapshot.read_reserve)
//         } else {
//             Err(())
//         }
//     }
//     pub fn read_commit(&self) {
//         todo!()
//     }
//     pub fn write_reserve(&self) {
//         todo!()
//     }
//     pub fn write_commit(&self) {
//         todo!()
//     }
// }

// struct CursorSnapshot<'a> {
//     cursor: &'a AtomicCursor,
//     read_commit: u8,
//     read_reserve: u8,
//     write_commit: u8,
//     write_reserve: u8,
//     grey: GreyCode,
//     dirty: bool,
// }

// impl<'a> CursorSnapshot<'a> {
//     pub fn from(cursor: &AtomicCursor) -> CursorSnapshot {
//         // todo: determine ordering
//         let n = cursor.inner.load(Ordering::Relaxed);
//         let bytes = u64::to_le_bytes(n);
//         let read_commit = bytes[0];
//         let read_reserve = bytes[1];
//         let write_commit = bytes[2];
//         let write_reserve = bytes[3];
//         let high: [u8; 4] = bytes[4..8].try_into().unwrap();
//         let grey = GreyCode(u32::from_le_bytes(high));
//         let dirty = false;
//         CursorSnapshot {
//             cursor,
//             read_commit,
//             read_reserve,
//             write_commit,
//             write_reserve,
//             grey,
//             dirty,
//         }
//     }

//     fn increment_reader(&mut self) -> () {
//         todo!()
//     }

//     fn try_commit(&mut self) -> bool {
//         todo!()
//     }
// }

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
