pub mod base;
pub mod flex;

use oxc_allocator::Allocator;

pub struct MemoryMachine<'alloc> {
    allocator: &'alloc Allocator,
}

impl<'a> MemoryMachine<'a> {
    async fn new(allocator: &'a Allocator) -> MemoryMachine<'a> {
        Self { allocator }
    }
}
