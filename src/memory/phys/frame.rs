/// 4KB binary order
pub const FRAME_SIZE_BITS: usize = 12;
/// 4KB
pub const FRAME_SIZE: usize = 1 << FRAME_SIZE_BITS;

/// Represents a physical frame in memory.
/// it must use the buddy system to manage memory. so to save memory block info is added here:
/// * `prev_block_head`: Pointer to the previous block head frame in the same order.
/// * `self_block_head`: Pointer to the current block head frame
/// * `next_block_head`: Pointer to the next block head frame in the same order.
pub struct Frame {
    pfn: usize,
    prev_block_head: Option<&'static Frame>,
    self_block_head: Option<&'static Frame>,
    next_block_head: Option<&'static Frame>,
}

impl Frame {
    pub fn new(pfn: usize) -> Self {
        Frame {
            pfn,
            prev_block_head: None,
            self_block_head: None,
            next_block_head: None,
        }
    }
}
