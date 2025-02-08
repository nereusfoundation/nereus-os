use core::ptr::NonNull;

use crate::{
    align_up, error::HeapError, paging::ptm::PageTableManager, VirtualAddress,
    KHEAP_PAGE_COUNT_MAX, PAGE_SIZE,
};

#[derive(Debug)]
pub struct ListNode {
    size: usize,
    free: bool,
    next: Option<NonNull<ListNode>>,
    prev: Option<NonNull<ListNode>>,
}

impl ListNode {
    pub fn free(&mut self) {
        self.free = true;
    }
}

#[derive(Debug)]
pub struct LinkedListAllocator {
    heap_size: usize,
    heap_start: VirtualAddress,
    head: Option<NonNull<ListNode>>,
}

impl LinkedListAllocator {
    /// Attempts to initialize new linked list allocator.
    ///
    /// # Safety
    /// Caller must ensure that the entire heap address space is valid and mapped.
    pub unsafe fn try_new(heap_start: VirtualAddress, heap_size: usize) -> Result<Self, HeapError> {
        if heap_size < size_of::<ListNode>() {
            Err(HeapError::InvalidBlockSize(heap_size))
        } else {
            let start_node = unsafe { NonNull::new_unchecked(heap_start as *mut ListNode) };

            // initialize start node that spans over the entire heap size
            unsafe {
                start_node.write(ListNode {
                    size: heap_size - size_of::<ListNode>(),
                    free: true,
                    next: None,
                    prev: None,
                });
            }
            Ok(Self {
                heap_size,
                heap_start,
                head: Some(start_node),
            })
        }
    }
}

impl LinkedListAllocator {
    /// Tries to find a fitting list node in the linked list to home a new block of allocated memory.
    pub fn find_fit(&mut self, size: usize) -> Result<NonNull<ListNode>, HeapError> {
        let mut current = self.head;
        while let Some(node) = current {
            let node_ref = unsafe { node.as_ref() };
            if node_ref.free && node_ref.size >= size {
                return Ok(node);
            }
            current = node_ref.next;
        }
        // no fit can be found (out of memory)
        Err(HeapError::Oom)
    }

    /// Splits a list node into two in order to allocate new memory on the heap. May fail if the size if too large.
    ///
    /// # Safety
    /// Caller must ensure the list node is valid.
    pub unsafe fn split_block(
        &mut self,
        mut node: NonNull<ListNode>,
        size: usize,
    ) -> Result<(), HeapError> {
        let node_ref = unsafe { node.as_mut() };
        let remaining_size = node_ref
            .size
            .checked_sub(size)
            .ok_or(HeapError::InvalidBlockSize(node_ref.size))?;

        if remaining_size >= size_of::<ListNode>() {
            let new_node_ptr = align_up(
                node.as_ptr() as u64 + (size_of::<ListNode>() + size) as u64,
                align_of::<ListNode>(),
            ) as *mut ListNode;

            let new_node = unsafe { NonNull::new_unchecked(new_node_ptr) };

            unsafe {
                new_node.write(ListNode {
                    size: remaining_size - size_of::<ListNode>(),
                    free: true,
                    next: node_ref.next,
                    prev: Some(node),
                });
            }

            if let Some(mut next_node) = node_ref.next {
                unsafe {
                    next_node.as_mut().prev = Some(new_node);
                }
            }

            node_ref.next = Some(new_node);
            node_ref.size = size;
        } else {
            // if remaining size is too small to split, just use the whole block instead of
            // splitting
            node_ref.size = remaining_size + size;
        }

        node_ref.free = false;

        Ok(())
    }

    /// Merges two list nodes. Used when freeing memory.
    ///
    /// # Safety
    /// Caller has to ensure that `node` points to a valid `ListNode`.
    pub unsafe fn merge_blocks(&mut self, mut node: NonNull<ListNode>) {
        let node_ref = unsafe { node.as_mut() };

        // merge with next node if it's free
        if let Some(mut next_node) = node_ref.next {
            let next_node_ref = unsafe { next_node.as_mut() };
            if next_node_ref.free {
                node_ref.size += next_node_ref.size + size_of::<ListNode>();
                node_ref.next = next_node_ref.next;

                if let Some(mut next_next_node) = next_node_ref.next {
                    unsafe {
                        next_next_node.as_mut().prev = Some(node);
                    }
                }
            }
        }

        // merge with previous node if it's free
        if let Some(mut prev_node) = node_ref.prev {
            let prev_node_ref = unsafe { prev_node.as_mut() };
            if prev_node_ref.free {
                prev_node_ref.size += node_ref.size + size_of::<ListNode>();
                prev_node_ref.next = node_ref.next;

                if let Some(mut next_node) = node_ref.next {
                    unsafe {
                        next_node.as_mut().prev = Some(prev_node);
                    }
                }
            }
        }
    }

    /// Attempts to expand the memory mapped for the heap allocator.
    pub fn expand(&mut self, size: usize, ptm: &mut PageTableManager) -> Result<(), HeapError> {
        let old_heap_page_count = self.heap_size.div_ceil(PAGE_SIZE);
        let new_heap_page_count = size.div_ceil(PAGE_SIZE) + old_heap_page_count;

        let flags = ptm.nx_flags();

        // check if expansion is valid
        if new_heap_page_count > KHEAP_PAGE_COUNT_MAX {
            return Err(HeapError::Oom);
        }
        for page in old_heap_page_count..new_heap_page_count {
            // allocate new physical frames for heap
            let physical_address = ptm.pmm().request_page().map_err(|_| HeapError::Oom)?;

            // map newly allocated frames to virtual heap offset
            ptm.map_memory(
                self.heap_start + (page * PAGE_SIZE) as u64,
                physical_address,
                flags,
            )
            .map_err(|_| HeapError::Oom)?;
        }

        // find last free list node and expand it
        let current = self.head;
        while let Some(mut node) = current {
            let node_ref = unsafe { node.as_mut() };
            // last free node
            if node_ref.free && node_ref.next.is_none() {
                node_ref.size += size;
                break;
            }
        }

        self.heap_size += size;

        Ok(())
    }
}
