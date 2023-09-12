#![no_std]

pub mod byte;
pub mod page;
pub mod static_page;

trait Allocator {
    fn alloc();
    fn dealloc();
    fn zalloc();
}
