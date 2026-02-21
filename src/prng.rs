//! 伪随机数生成器模块
//!
//! 本模块提供了用于生成确定性随机序列的工具。
//! 在图像混淆应用中，我们需要能够使用相同的种子（seed）生成相同的随机序列，
//! 以便能够正确地进行混淆和解混淆操作。
//!
//! ## 算法选择
//!
//! 本项目使用 SplitMix64 算法，这是一种高质量的 64 位伪随机数生成器：
//! - 速度快：仅包含几次位运算和乘法
//! - 质量高：通过了许多随机性测试
//! - 确定性：相同的种子产生相同的序列
//! - 非黄金分割：与常见的线性同余生成器不同，不依赖黄金比例常数
//!
//! ## 核心应用
//!
//! PRNG 在本项目中的主要用途是实现 Fisher-Yates 洗牌算法，
//! 用于随机打乱像素数组的顺序

/// SplitMix64 伪随机数生成器
///
/// 这是一种基于 Vigna 教授设计的算法（SplitMix64）的实现。
/// "SplitMix" 名称来源于算法的核心结构：将状态分割并进行混合操作。
///
/// ## 算法原理
///
/// SplitMix64 的核心是一个 64 位状态寄存器，通过以下步骤生成随机数：
/// 1. 状态更新：state += 0x9E3779B97F4A7C15（一个随机性很强的常数）
/// 2. 第一次混合：z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9
/// 3. 第二次混合：z = (z ^ (z >> 27)) * 0x94D049BB133111EB
/// 4. 最终扰动：z ^ (z >> 31)
///
/// 这些常数和移位/乘法操作的组合确保了输出的良好统计特性。
///
/// ## 特性说明
/// - 周期：2^64（因为使用 64 位状态且每次更新都是可逆的）
/// - 速度：非常快，只需要几次位运算
/// - 确定性：相同初始状态产生相同的输出序列
#[derive(Clone, Debug)]
pub struct SplitMix64 {
    /// 内部状态寄存器，存储当前的随机数生成器状态
    state: u64,
}

impl SplitMix64 {
    /// 使用给定的种子创建新的 SplitMix64 生成器
    ///
    /// # 参数说明
    /// - `seed`: 初始种子值，用于初始化内部状态
    ///
    /// # 返回值
    /// 返回一个新的 SplitMix64 实例
    pub fn new(seed: u64) -> Self {
        SplitMix64 { state: seed }
    }

    /// 生成下一个 64 位随机数
    ///
    /// 每次调用都会更新内部状态并产生一个新的随机数。
    /// 这是 SplitMix64 算法的核心函数。
    ///
    /// # 返回值
    /// 返回一个 64 位的无符号整数作为随机数
    ///
    /// # 算法细节
    /// 1. 首先将状态加上常数 0x9E3779B97F4A7C15 进行更新
    /// 2. 进行三次混合变换，使用异或和乘法操作
    /// 3. 最后返回混合后的值
    pub fn next(&mut self) -> u64 {
        let mut z = {
            self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
            self.state
        };
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    /// 生成指定范围内的随机数 [0, bound)
    ///
    /// 这是一个便捷方法，用于生成适合数组索引的随机数。
    /// 使用简单的取模操作，效率较高。
    ///
    /// # 参数说明
    /// - `bound`: 上界（不包含），必须是正整数
    ///
    /// # 返回值
    /// 返回 [0, bound) 范围内的随机数
    ///
    /// # 注意事项
    /// 由于使用简单的取模，当 bound 不是 2 的幂次时，
    /// 输出的分布会有轻微偏差，但对于本应用（洗牌算法）来说可以接受
    pub fn next_bound(&mut self, bound: usize) -> usize {
        (self.next() as usize) % bound
    }
}

/// 生成索引数组的随机排列（洗牌）
///
/// 使用 Fisher-Yates（又称 Knuth）洗牌算法，结合 SplitMix64 伪随机数生成器，
/// 生成一个确定性的索引排列。
///
/// # 算法原理
///
/// Fisher-Yates 洗牌算法是一个经典的洗牌算法，其基本思想是：
/// 1. 从最后一个位置开始向前遍历
/// 2. 对于每个位置 i，在 [0, i] 范围内随机选择一个位置 j
/// 3. 交换位置 i 和 j 的元素
///
/// 这种算法确保：
/// - 每个可能的排列出现的概率相等
/// - 时间复杂度为 O(n)
/// - 空间复杂度为 O(n)（仅需要存储索引数组）
///
/// # 参数说明
/// - `n`: 要生成的索引数量，生成 [0, 1, 2, ..., n-1] 的随机排列
/// - `seed`: 随机种子，用于初始化 SplitMix64 生成器
///
/// # 返回值
/// 返回一个 Vec<usize>，包含 0 到 n-1 的所有索引，但顺序是随机打乱的
///
/// # 重要特性
///
/// **确定性**：相同的 seed 和 n 会产生完全相同的排列
/// 这对于图像混淆/解混淆非常重要，因为我们需要在混淆和解混淆时使用相同的排列
pub fn shuffle_indices(n: usize, seed: u64) -> Vec<usize> {
    let mut rng = SplitMix64::new(seed);
    let mut indices: Vec<usize> = (0..n).collect();

    // Fisher-Yates shuffle
    for i in (1..n).rev() {
        let j = rng.next_bound(i + 1);
        indices.swap(i, j);
    }

    indices
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试 SplitMix64 的确定性
    ///
    /// 验证相同的种子能够产生完全相同的随机数序列
    #[test]
    fn test_splitmix64_deterministic() {
        let mut rng1 = SplitMix64::new(12345);
        let mut rng2 = SplitMix64::new(12345);

        for _ in 0..100 {
            assert_eq!(rng1.next(), rng2.next());
        }
    }

    /// 测试不同种子产生不同的随机数序列
    #[test]
    fn test_different_seeds() {
        let mut rng1 = SplitMix64::new(1);
        let mut rng2 = SplitMix64::new(2);

        let v1 = rng1.next();
        let v2 = rng2.next();

        assert_ne!(v1, v2);
    }

    /// 测试 shuffle_indices 的确定性
    ///
    /// 验证相同的参数产生相同的洗牌结果
    #[test]
    fn test_shuffle_deterministic() {
        let perm1 = shuffle_indices(100, 12345);
        let perm2 = shuffle_indices(100, 12345);

        assert_eq!(perm1, perm2);
    }

    /// 测试洗牌结果包含所有原始索引
    ///
    /// 验证洗牌是一个有效的排列，不丢失任何元素
    #[test]
    fn test_shuffle_contains_all() {
        let perm = shuffle_indices(50, 42);

        let mut sorted = perm.clone();
        sorted.sort();
        assert_eq!(sorted, (0..50).collect::<Vec<_>>());
    }

    /// 测试不同种子产生不同的洗牌结果
    #[test]
    fn test_shuffle_different_seeds() {
        let perm1 = shuffle_indices(100, 1);
        let perm2 = shuffle_indices(100, 2);

        assert_ne!(perm1, perm2);
    }
}
