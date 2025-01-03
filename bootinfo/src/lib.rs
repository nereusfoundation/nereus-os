#![no_std]

use framebuffer::raw::write::RawWriter;
use mem::{map::MemoryMap, paging::ptm::PageTableManager};

#[derive(Debug)]
pub struct BootInfo {
    pub mmap: MemoryMap,
    pub writer: Option<RawWriter>,
    pub ptm: PageTableManager,
    pub nx: bool,
}
