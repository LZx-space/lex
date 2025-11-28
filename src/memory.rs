use core::{
    num::NonZeroUsize,
    sync::atomic::{AtomicUsize, Ordering},
};

use spin::lazy::Lazy;

pub mod phys;

pub type PhysicalAddress = NonZeroUsize;

pub fn init() -> Result<(), &'static str> {
    phys::init()
}

/////////////////////////////////////////////////////////////////
// BOOT阶段内存帧分配器
/////////////////////////////////////////////////////////////////
unsafe extern "C" {
    static HEAP_START: usize;
    static HEAP_SIZE: usize;
}

/// 简易boot阶段内存帧分配器实例
pub static BOOT_FRAME_ALLOCATOR: Lazy<BootFrameAllocator> = Lazy::new(|| unsafe {
    let from = PhysicalAddress::new_unchecked(HEAP_START);
    let to = PhysicalAddress::new_unchecked(HEAP_START + HEAP_SIZE);
    BootFrameAllocator::new(from, to)
});

/// 简易boot阶段内存帧分配器
pub struct BootFrameAllocator {
    next_addr: AtomicUsize,
    start_addr: PhysicalAddress,
    end_addr: PhysicalAddress,
}

#[allow(unused)]
impl BootFrameAllocator {
    pub fn new(start: PhysicalAddress, end: PhysicalAddress) -> Self {
        Self {
            next_addr: AtomicUsize::new(start.get()),
            start_addr: start,
            end_addr: end,
        }
    }

    /// 返回一个按对齐模向上对齐的内存地址
    pub fn alloc(&self, size: usize, align: usize) -> Option<PhysicalAddress> {
        let align = align.next_power_of_two();
        let align_mask = align - 1;

        loop {
            let current = self.next_addr.load(Ordering::Acquire);
            let aligned_addr = (current + align_mask) & !(align_mask);
            let new_next = aligned_addr + size;

            if new_next > self.end_addr.get() {
                return None; // 内存不足
            }

            // 尝试原子地更新分配位置
            match self.next_addr.compare_exchange_weak(
                current,
                new_next,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => return unsafe { Some(PhysicalAddress::new_unchecked(aligned_addr)) },
                Err(_) => continue, // 被其他分配干扰，重试
            }
        }
    }

    pub fn allocated(&self) -> usize {
        self.next_addr.load(Ordering::Relaxed) - self.start_addr.get()
    }

    pub fn remaining(&self) -> usize {
        self.end_addr.get() - self.next_addr.load(Ordering::Relaxed)
    }
}
