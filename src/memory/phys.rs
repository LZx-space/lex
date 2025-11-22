pub mod buddy;
pub mod frame;
pub mod sparse;

use crate::memory::phys::sparse::sparse_init;
use crate::println;

/////////////////////////////////////////////////////////////
// 初始化
/////////////////////////////////////////////////////////////

/// 按此顺序划分区域
/// ```
/// 低位地址 -------------------高位地址
/// N * 每个物理页状态 | 实际可分配的物理页
/// ```
/// 旁注：这不是一个精准的算法，待优化
pub fn init() {
    println!("物理内存管理初始化-开始");
    match sparse_init() {
        Ok(_) => {}
        Err(err) => {
            println!("稀疏物理内存管理初始化失败：{}", err);
        }
    }
}
