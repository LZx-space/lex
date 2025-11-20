/// Sparse Memory Model Implementation
use spin::mutex::Mutex;

use crate::memory::phys::frame::{FRAME_SIZE, FRAME_SIZE_BITS, Frame};
use crate::memory::{BOOT_FRAME_ALLOCATOR, PhysicalAddress};
use crate::util::{align_down_pow2, align_up_pow2};
use core::mem::size_of;
use core::ptr::NonNull;
use core::slice::from_raw_parts_mut;

/// 128MB binary order
pub const SECTION_SIZE_BITS: usize = 27;
/// 128MB
pub const SECTION_SIZE: usize = 1 << SECTION_SIZE_BITS;
/// 2^15 = 32768
pub const PAGES_PER_SECTION: usize = SECTION_SIZE / FRAME_SIZE;

/// 可能的可分配物理内存区域
/// # 图示
/// |0x0000     |           |           |0xffff     |
/// |-----------|-----------|-----------|-----------|
/// |taken      |free       |taken      |free       |
/// |           |n*section  |           |n*section  |
pub struct MemorySection {
    /// Pointer to this section's frame array
    mem_frames: Option<NonNull<Frame>>,
    num_frames: usize,
}

impl MemorySection {
    pub fn new() -> Self {
        Self {
            mem_frames: None,
            num_frames: 0,
        }
    }

    /// Initialize the section with physical memory
    /// This allocates the frame array for this section
    pub fn init(&mut self, num_frames: usize) -> Result<(), &'static str> {
        if num_frames > PAGES_PER_SECTION {
            return Err("Section overflow: too many pages");
        }
        // Allocate memory for frame structures
        let frame_array_size = num_frames * size_of::<Frame>();
        let frame_array_ptr = BOOT_FRAME_ALLOCATOR
            .alloc(frame_array_size, FRAME_SIZE)
            .ok_or("no more memory")?
            .get();

        // Initialize all frame structures
        let frame_array = unsafe { from_raw_parts_mut(frame_array_ptr as *mut Frame, num_frames) };
        for frame in frame_array {
            *frame = Frame {};
        }
        self.mem_frames = Some(NonNull::new(frame_array_ptr as *mut Frame).unwrap());
        self.num_frames = num_frames;
        Ok(())
    }

    /// # 参数
    /// * `frame_offset` - 帧在`MemorySection::mem_pages`中的偏移量(数组索引)
    pub fn get_frame(&self, frame_offset: usize) -> Option<&Frame> {
        if let Some(ptr) = self.mem_frames {
            // 需要存储实际的帧数量来进行边界检查
            // 这里假设有 num_frames 字段
            if frame_offset < self.num_frames {
                unsafe {
                    let frame_ptr = ptr.as_ptr().add(frame_offset);
                    Some(&*frame_ptr)
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Sparse memory manager
///////////////////////////////////////////////////////////////////////////////

unsafe extern "C" {
    static HEAP_START: usize;
    static HEAP_SIZE: usize;
}

/// Global sparse memory manager instance
/// Global sparse memory manager
static SPARSE_MEMORY_MANAGER: Mutex<SparseMemoryManager> = Mutex::new(SparseMemoryManager::new());

/// 提供线程安全的访问方法
#[allow(unused)]
pub fn get_sparse_manager() -> &'static Mutex<SparseMemoryManager> {
    &SPARSE_MEMORY_MANAGER
}
/// Initialize sparse memory system
pub fn sparse_init() -> Result<(), &'static str> {
    unsafe {
        let region_start = PhysicalAddress::new_unchecked(HEAP_START);
        let region_end = PhysicalAddress::new_unchecked(HEAP_START + HEAP_SIZE);
        SPARSE_MEMORY_MANAGER
            .lock()
            .init_memory_region(region_start, region_end)?;
    }
    Ok(())
}

/// Maximum number of sections supported
/// For 64-bit system with 128MB sections, this supports up to 2^47 bytes (128TB)
pub const MAX_SECTIONS: usize = 1 << (47 - SECTION_SIZE_BITS);

/// Global sparse memory manager
pub struct SparseMemoryManager {
    /// Array of memory sections
    /// Uses Option to handle sparse allocation
    sections: [Option<MemorySection>; MAX_SECTIONS],

    /// Total section has been initialized
    total_sections: usize,

    /// Total number of pages in the system
    total_frames: usize,
}

// ==============================================================================
// 核心修改：手动为 SparseMemoryManager 实现 Send + Sync
// 安全前提：所有对 SparseMemoryManager 的访问都被全局自旋锁保护，无并发竞争
// ==============================================================================
unsafe impl Send for SparseMemoryManager {}
unsafe impl Sync for SparseMemoryManager {}

impl SparseMemoryManager {
    /// Create a new sparse memory manager
    const fn new() -> Self {
        Self {
            sections: [const { None }; MAX_SECTIONS],
            total_sections: 0,
            total_frames: 0,
        }
    }

    /// Initialize a memory region
    /// This is called during boot for each contiguous memory region
    pub fn init_memory_region(
        &mut self,
        region_start: PhysicalAddress,
        region_end: PhysicalAddress,
    ) -> Result<(), &'static str> {
        let region_start = align_up_pow2(region_start.get(), FRAME_SIZE_BITS);
        let region_end = align_down_pow2(region_end.get(), FRAME_SIZE_BITS);
        if region_end <= region_start {
            return Err("Invalid memory region: end <= start after alignment");
        }

        // Initialize each section in the region
        let section_ranges = (region_start..=region_end)
            .step_by(SECTION_SIZE)
            .map(|start| (start, start + SECTION_SIZE - 1))
            .take_while(|&(_, end)| end <= region_end);

        for (section_num, (start, end)) in section_ranges.enumerate() {
            if self.total_sections >= MAX_SECTIONS {
                return Err("Too many memory sections: exceeded MAX_SECTIONS");
            }
            let frames_in_section = (end - start + 1) / FRAME_SIZE;

            let mut section = MemorySection::new();
            section.init(frames_in_section)?;

            self.sections[section_num] = Some(section);
            self.total_sections += 1;
            self.total_frames += frames_in_section;
        }
        Ok(())
    }
}
