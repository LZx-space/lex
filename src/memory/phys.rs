use crate::{println, util};
use core::cmp::max;
use core::fmt::Error;
use core::num::NonZeroUsize;
use heapless::Vec;

unsafe extern "C" {
    static HEAP_START: usize;
    static HEAP_SIZE: usize;
}

/// 页大小
const FRAME_SIZE: usize = 1 << FRAME_SIZE_ORDER;
/// 页大小对应的2进制的阶数
const FRAME_SIZE_ORDER: usize = 12;
/////////////////////////////////////////////////////////////
// 初始化
/////////////////////////////////////////////////////////////

/// 按此顺序划分区域
/// ```
/// 低位地址 -------------------高位地址
/// N * 每个物理页状态 | 实际可分配的物理页
/// ```
/// 旁注：这不是一个精准的算法，待优化
pub fn init() {}

//////////////////////////////////////////////////////////////
// 内存帧，物理内存最小使用单位
//////////////////////////////////////////////////////////////

type PhysicalAddress = NonZeroUsize;

// -------------------------------------------------------------------------------------------------
/// 可被分帧的内存块
/// * 空闲内存可能不连续，尤其是OS启动时
/// * 尽可能地获取这些内存块来分帧以提高内存利用率
pub struct MemoryRegion {
    aligned_from: PhysicalAddress,
    aligned_to: PhysicalAddress,
}

impl MemoryRegion {
    pub fn new(from: usize, to: usize) -> Result<MemoryRegion, Error> {
        let from = util::align_up_pow2(from, FRAME_SIZE_ORDER);
        let to = util::align_down_pow2(to, FRAME_SIZE_ORDER);
        if from == 0 || to == 0 {
            return Err(Error);
        }
        let from = PhysicalAddress::new(from).expect("bad from addr");
        let to = PhysicalAddress::new(to).expect("bad to addr");
        Ok(MemoryRegion {
            aligned_from: from,
            aligned_to: to,
        })
    }
}
/// 可能的可分配物理内存区域
/// # 图示
/// ```
/// 低位地址------------------------------------------高位地址
/// 0x0000---已占---|---可分帧---|---已占---|---可分帧---0xffff
/// ```
/// 暂且假设只有一个
fn memory_region() -> MemoryRegion {
    unsafe { MemoryRegion::new(HEAP_START, HEAP_START + HEAP_SIZE).expect("") }
}

/// 相关术语
/// * 帧：物理内存中固定大小的块
/// * 页：虚拟内存中固定大小的块
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Frame {
    /// 该帧首物理地址
    addr: PhysicalAddress,
}

impl Frame {
    /// 创建帧，地址会按[`FRAME_SIZE`]对齐模数找到首地址
    /// # 示例
    /// ```
    /// let frame = Frame::new(PhysicalAddress::new(0x1000)).unwrap();
    /// assert_eq!(frame.addr(), PhysicalAddress::new(0x1001));
    /// ```
    pub fn new(address: PhysicalAddress) -> Result<Frame, Error> {
        if address.get().lt(&FRAME_SIZE) {
            return Err(Error);
        }
        let base_addr = util::align_down_pow2(address.into(), FRAME_SIZE_ORDER);
        PhysicalAddress::new(base_addr)
            .map(|addr| Frame { addr })
            .ok_or_else(|| Error)
    }

    /// 帧大小
    pub fn frame_size() -> usize {
        FRAME_SIZE
    }
}

//////////////////////////////////////////////////////////////
// Buddy System - 伙伴系统算法实现，物理内存管理
//////////////////////////////////////////////////////////////

/// 伙伴系统算法中最大阶数
const MAX_BUDDY_ORDER: usize = 10;

/// 伙伴系统算法中最大阶数对应的阶数个数，11即0-10，而2^10 = 1024
const NUM_BUDDY_ORDERS: usize = MAX_BUDDY_ORDER + 1;

// -------------------------------------------------------------------------------------------------
/// 伙伴系统中一个Order对应N个该数据结构，它代表多个地址连续的[`Frame`]
/// # 属性
/// * [`first`] - 首个[Frame]
/// * [`binary_order`] - 伙伴系统中其所属二进制阶数
pub struct FrameBlock {
    first: Frame,
    binary_order: usize,
}

impl FrameBlock {
    pub fn new(first: Frame, binary_order: usize) -> FrameBlock {
        FrameBlock {
            first,
            binary_order,
        }
    }

    pub fn addr_range_inclusive(&self) -> (PhysicalAddress, PhysicalAddress) {
        let addr = self.first.addr.get() + (1 + 1 << self.binary_order) * FRAME_SIZE;
        (
            self.first.addr,
            PhysicalAddress::new(addr).expect("never fail"),
        )
    }
}

/// 将所有的[`MemoryRegion`]拆分到伙伴系统的元素[`FrameBlock`]，以高Order优先的方式填充伙伴系统
/// # 伙伴系统（算法）
/// * 查阅文档
/// * 算法中一个2的Order次方对应N个的[`FrameBlock`]，[`FrameBlock`]大小就为2^order个单位
/// # 属性
/// * [`order_indexed_block`] - 为固定长度[`NUM_BUDDY_ORDERS`]的数组，元素类型为[`Option<FrameBlock>`]以可表被分配，
///   [`FrameBlock`]由多个地址相接的[`Frame`]组成
struct BuddySystem {
    order_indexed_block: Vec<Option<FrameBlock>, NUM_BUDDY_ORDERS>,
}
#[allow(unused)]
impl BuddySystem {
    /// # 参数
    /// * [`region`]-地址连续的物理内存区域
    /// # 返回值
    /// * 该物理内存区域对应的伙伴系统
    pub fn new(region: MemoryRegion) -> Self {
        let mut order_indexed_block: Vec<Option<FrameBlock>, NUM_BUDDY_ORDERS> = Default::default();
        let region_addr_from = region.aligned_from.get();
        let region_addr_to = region.aligned_to.get();

        let region_byte_len = region_addr_from - region_addr_to;
        let mut region_frames = region_byte_len / FRAME_SIZE;
        let max_order = region_frames.ilog2();
        let order = max(max_order as usize, NUM_BUDDY_ORDERS);
        // 初始化所有frames
        let mut frames: Vec<Frame, { region_frames }> = Vec::new();
        (region_addr_from..region_addr_to)
            .step_by(FRAME_SIZE)
            .map(|chunk_start| {
                let chunk_end = (chunk_start + FRAME_SIZE);
                println!("frame addr: {}..{}", chunk_start, chunk_end);
                let frame = Frame::new(PhysicalAddress::try_from(chunk_start).unwrap()).unwrap();
                frames.push(frame);
            });
        // A le NUM_ORDERS:
        //      找出order_indexed_block下标为A的元素，添加实际为所有该Section的Frame构成的FrameBlock
        //      余数尽量拆分为数量最少阶数更高的数，插入对应下标的元素
        // A gt NUM_ORDERS:
        //      尽量拆分为数量最少阶数更高的数，插入对应下标的元素
        let num_frames = frames.len();
        while order > 0 {
            let num_order_frames = 2.pow(order as u32);
            let frame_block = FrameBlock::new(frames[num_frames - num_order_frames], order);
            order_indexed_block.insert(order, Some(frame_block));
        }
        BuddySystem {
            order_indexed_block,
        }
    }

    /// 分配指定2^binary_order大小的内存，分配的[`FrameBlock`]至少大于所需内存，当无法分配时返回异常
    fn allocate(&mut self, binary_order: usize) -> Result<FrameBlock, Error> {
        todo!()
    }

    /// 释放指定[`FrameBlock`]
    fn deallocate(&mut self, block: FrameBlock) -> Result<(), Error> {
        todo!()
    }
}

//////////////////////////////////////////////////////////////
// todo 多个内存区域建立多个伙伴系统，多个系统如何调度使用的相关设计
//      当前仅按单个伙伴系统
//////////////////////////////////////////////////////////////
