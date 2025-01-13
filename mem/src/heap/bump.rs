// todo: add this as a feature only

use core::{alloc::Layout, ptr};

use crate::{heap::align_up, VirtualAddress};

#[derive(Copy, Clone, Debug)]
pub struct BumpAllocator {
    heap_start: VirtualAddress,
    heap_end: VirtualAddress,
    next: VirtualAddress,
    allocations: usize,
}

impl Default for BumpAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    /// Initialize the bump allocator with the given address.
    ///
    /// # Safety
    /// Caller must guarantee that the pages for the heap are mapped.
    pub unsafe fn init(&mut self, heap_start: VirtualAddress, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_size as VirtualAddress + heap_start;
        self.next = heap_start;
    }
}

impl BumpAllocator {
    /// Allocate memory according to the specified layout
    ///
    /// # Safety
    /// layout must have non-zero size. Attempting to allocate for a zero-sized layout may result in undefined behavior.
    ///
    /// (Extension subtraits might provide more specific bounds on behavior, e.g., guarantee a sentinel address or a null pointer in response to a zero-size allocation request.)
    /// The allocated block of memory may or may not be initialized.
    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let alloc_start = align_up(self.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size() as VirtualAddress) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > self.heap_end {
            // out of memory :(
            ptr::null_mut()
        } else {
            self.next = alloc_end;
            self.allocations += 1;
            alloc_start as *mut u8
        }
    }

    /// Deallocate the memory at the specified pointer according to the specified layout.
    ///
    /// # Safety
    ///
    /// The caller has to ensure that `layout` has non-zero size. Like `alloc`
    /// zero sized `layout` can result in undefined behavior.
    /// However the allocated block of memory is guaranteed to be initialized.
    pub unsafe fn dealloc(&mut self, _ptr: *mut u8, _layout: Layout) {
        self.allocations -= 1;

        if self.allocations == 0 {
            self.next = self.heap_start;
        }
    }
}
