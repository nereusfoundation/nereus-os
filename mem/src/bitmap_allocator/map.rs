use core::slice;

use crate::{error::FrameAllocatorError, PAGE_SIZE};

#[repr(transparent)]
#[derive(Debug)]
pub struct BitMap {
    buffer: &'static mut [u8],
}

impl BitMap {
    pub(crate) fn new(buffer: &'static mut [u8]) -> BitMap {
        BitMap { buffer }
    }
}

impl BitMap {
    /// Update the bitmap buffer pointer
    ///
    /// # Safety
    /// The caller must guarantee that the pointer is valid.
    pub(crate) unsafe fn update_ptr(&mut self, ptr: *mut u8) {
        let len = self.buffer.len();
        self.buffer = unsafe { slice::from_raw_parts_mut(ptr, len) };
    }

    /// Retrieve raw pointer to bitmap
    ///
    /// # Safety
    /// Caller gets control of raw pointer to bitmap - must be handled with care
    pub(crate) unsafe fn ptr(&mut self) -> *mut u8 {
        self.buffer.as_mut_ptr() as *mut u8
    }
}

impl BitMap {
    /// Get the bit on a certain index (in bits)
    pub fn get(&self, index: u64) -> Result<bool, FrameAllocatorError> {
        let byte_index = index / 8;
        if byte_index >= self.buffer.len() as u64 {
            return Err(FrameAllocatorError::InvalidBitMapIndex);
        }
        let bit_index = index % 8;
        let bit_indexer = 0b10000000 >> bit_index;
        Ok((self.buffer[byte_index as usize] & bit_indexer) != 0)
    }

    /// Set the bit on a certain index (in bits)
    pub fn set(&mut self, index: u64, value: bool) -> Result<(), FrameAllocatorError> {
        let byte_index = index / 8;
        if byte_index >= self.buffer.len() as u64 {
            return Err(FrameAllocatorError::InvalidBitMapIndex);
        }
        let bit_index = index % 8;

        let bit_indexer = 0b10000000 >> bit_index;

        // set index to false
        self.buffer[byte_index as usize] &= !bit_indexer;

        if value {
            self.buffer[byte_index as usize] |= bit_indexer;
        }

        Ok(())
    }

    pub fn pages(&self) -> usize {
        self.buffer.len().div_ceil(PAGE_SIZE)
    }
}
