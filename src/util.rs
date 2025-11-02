/// 向上对齐
/// * `val` - 被对齐数
/// * `order` - 对齐模为2的`order`次方
/// # 示例：
/// ```
/// let aligned = align_up_to_power_of_two(100, 4);
/// assert_eq!(aligned, 112);
/// ```
pub const fn align_up_pow2(val: usize, order: usize) -> usize {
    let modulus = 1usize << order;
    let mask = modulus - 1;
    (val + mask) & !mask
}

/// 向下对齐
/// * `val` - 被对齐数
/// * `order` - 对齐模为2的`order`次方
/// # 示例：
/// ```
/// let aligned = align_down_to_power_of_two(100, 4);
/// assert_eq!(aligned, 96);
/// ```
pub const fn align_down_pow2(val: usize, order: usize) -> usize {
    let modulus = 1usize << order;
    let mask = !(modulus - 1);
    val & mask
}
