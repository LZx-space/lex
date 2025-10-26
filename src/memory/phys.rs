use crate::println;

unsafe extern "C" {
    static HEAP_START: usize;
    static HEAP_SIZE: usize;
}

/// 页大小
const PAGE_SIZE: usize = 1 << PAGE_SIZE_ORDER;
/// 页大小对应的2进制的阶数
const PAGE_SIZE_ORDER: usize = 12;
static mut ALLOC_START: usize = 0;

/// 按此顺序划分区域
/// ```
/// 低位地址 -------------------高位地址
/// N * 每个物理页状态 | 实际可分配的物理页
/// ```
/// 旁注：这不是一个精准的算法，待优化
pub fn init() {
    unsafe {
        let num_pages = HEAP_SIZE / PAGE_SIZE;
        // 从HEAP_START地址开始的内存按PhysicalPage类型的布局解释
        let ptr = HEAP_START as *mut PhysicalPage;
        for i in 0..num_pages {
            (*ptr.add(i)).clear();
        }
        let pages_state_end = HEAP_START + num_pages * size_of::<PhysicalPage>();
        ALLOC_START = align_to_power_of_two(pages_state_end, PAGE_SIZE_ORDER);
        let tmp = ALLOC_START;
        println!("可分配的物理内存分页实际开始地址{}", tmp);
    }
}

/// 返回对齐后数字
/// * `order`为2的阶数，2的`order`次方即为对齐模数
/// * 如下示例：返回的对齐数为112，则下一个页的开始地址为112
/// ```
/// let aligned = align_to_power_of_two(100, 4);
/// assert_eq!(aligned, 112);
/// ```
pub const fn align_to_power_of_two(val: usize, order: usize) -> usize {
    let modulus = 1usize << order;
    let mask_complement = modulus - 1;
    (val + mask_complement) & !mask_complement
}

/// 物理内存分页二，每页4096字节
pub struct PhysicalPage {
    /// 该页的状态或是否为最后页的标识符[`PhysicalPageFlags`]
    flags: u8,
}

impl PhysicalPage {
    ///
    pub fn is_last(&self) -> bool {
        if self.flags & PhysicalPageFlags::Last.val() != 0 {
            true
        } else {
            false
        }
    }

    /// 当前页是否被分配
    pub fn is_used(&self) -> bool {
        if self.flags & PhysicalPageFlags::Used.val() != 0 {
            true
        } else {
            false
        }
    }

    /// 当前页是否空闲
    pub fn is_free(&self) -> bool {
        !self.is_used()
    }

    /// 清空状态（置于空闲态）
    pub fn clear(&mut self) {
        self.flags = PhysicalPageFlags::Free.val();
    }

    // Set a certain flag. We ran into trouble here since PageBits
    // is an enumeration, and we haven't implemented the BitOr Trait
    // on it.
    pub fn set_flag(&mut self, flag: PhysicalPageFlags) {
        self.flags |= flag.val();
    }

    pub fn clear_flag(&mut self, flag: PhysicalPageFlags) {
        self.flags &= !(flag.val());
    }
}

/// * 页的使用状态：空闲、已使用
/// * 是否为最后页
#[repr(u8)]
pub enum PhysicalPageFlags {
    Free = 0 << 0,
    Used = 1 << 0,
    Last = 1 << 1,
}

impl PhysicalPageFlags {
    pub fn val(self) -> u8 {
        self as u8
    }
}
