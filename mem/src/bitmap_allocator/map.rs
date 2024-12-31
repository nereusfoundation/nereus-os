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
        size_of::<BitMap>().div_ceil(PAGE_SIZE)
    }
}
