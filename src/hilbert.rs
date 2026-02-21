//! Hilbert 曲线实现模块
//!
//! 本模块提供了 Hilbert 曲线（又称空间填充曲线）的核心功能，
//! 用于在二维坐标和一维索引之间进行双向转换。
//!
//! ## Hilbert 曲线简介
//!
//! Hilbert 曲线是一种连续的空间填充曲线，由 David Hilbert 于 1891 年发现。
//! 它具有以下重要特性：
//! - 将二维空间映射到一维空间，同时保持局部性（相近的二维点在Hilbert曲线上也相近）
//! - 自相似的分形结构
//! - 在图像处理、数据压缩、空间索引等领域有广泛应用
//!
//! ## 算法原理
//!
//! 本实现使用 hilbert_2d crate 提供的离散 Hilbert 曲线算法：
//! - `hilbert_index()`: 将 (x, y) 坐标转换为 Hilbert 曲线上的索引位置
//! - `hilbert_inverse()`: 将 Hilbert 曲线索引反向转换为 (x, y) 坐标
//!
//! ## 应用场景
//!
//! 在图像混淆中，Hilbert 曲线用于：
//! 1. 将二维图像像素映射到一维数组（保持空间局部性）
//! 2. 打乱像素顺序但保留某种空间相关性
//! 3. 使得单纯的像素重排看起来更加"自然"

use hilbert_2d::{h2xy_discrete, xy2h_discrete, Variant};

/// 将二维坐标 (x, y) 转换为 Hilbert 曲线索引
///
/// # 参数说明
/// - `x`: 二维坐标的 X 轴分量（水平方向）
/// - `y`: 二维坐标的 Y 轴分量（垂直方向）
/// - `side`: 正方形区域的边长（必须是 2 的幂次方）
///
/// # 返回值
/// 返回一个 usize 类型的索引值，表示该坐标在 Hilbert 曲线上从起点开始的位置
pub fn hilbert_index(x: u32, y: u32, side: u32) -> usize {
    xy2h_discrete(x as usize, y as usize, side as usize, Variant::Hilbert)
}

/// 将 Hilbert 曲线索引反向转换为二维坐标 (x, y)
///
/// 这是 `hilbert_index` 函数的逆操作
///
/// # 参数说明
/// - `index`: Hilbert 曲线上的索引位置（从 0 开始）
/// - `side`: 正方形区域的边长（必须是 2 的幂次方）
///
/// # 返回值
/// 返回一个元组 (x, y)，表示对应的二维坐标
pub fn hilbert_inverse(index: usize, side: u32) -> (u32, u32) {
    let (x, y) = h2xy_discrete(index, side as usize, Variant::Hilbert);
    (x as u32, y as u32)
}

/// 将二维坐标 (x, y) 转换为 Hilbert 曲线的距离值 d
///
/// 这是 `hilbert_index` 函数的别名，提供更直观的命名
/// "d" 表示 distance，即曲线上的距离/位置
///
/// # 参数说明
/// - `n`: 正方形区域的边长（必须是 2 的幂次方）
/// - `x`: 二维坐标的 X 轴分量
/// - `y`: 二维坐标的 Y 轴分量
///
/// # 返回值
/// 返回 Hilbert 曲线上的距离值
///
/// # 数学含义
/// 在 Hilbert 曲线上，距离 d 表示从曲线起点到该点所经过的总长度
/// 对于 N×N 的网格，d 的取值范围是 [0, N*N-1]
pub fn xy2d(n: u32, x: u32, y: u32) -> usize {
    hilbert_index(x, y, n)
}

/// 将 Hilbert 曲线的距离值 d 转换为二维坐标 (x, y)
///
/// 这是 `hilbert_inverse` 函数的别名，提供更直观的命名
///
/// # 参数说明
/// - `n`: 正方形区域的边长（必须是 2 的幂次方）
/// - `d`: Hilbert 曲线上的距离值
///
/// # 返回值
/// 返回对应的二维坐标 (x, y)
pub fn d2xy(n: u32, d: usize) -> (u32, u32) {
    hilbert_inverse(d, n)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试 2x2 Hilbert 曲线的正确性
    ///
    /// 2x2 Hilbert 曲线的遍历顺序为：
    /// (0,0) -> 0 -> (0,0)
    /// (1,0) -> 1 -> (1,0)
    /// (1,1) -> 2 -> (1,1)
    /// (0,1) -> 3 -> (0,1)
    #[test]
    fn test_hilbert_2x2() {
        let expected = vec![(0, 0, 0), (1, 0, 1), (1, 1, 2), (0, 1, 3)];
        for (x, y, expected_d) in expected {
            let d = xy2d(2, x, y);
            assert_eq!(
                d, expected_d,
                "xy2d({}, {}, 2) = {}, expected {}",
                x, y, d, expected_d
            );

            let (rx, ry) = d2xy(2, expected_d);
            assert_eq!(
                (rx, ry),
                (x, y),
                "d2xy({}, 2) = ({}, {}), expected ({}, {})",
                expected_d,
                rx,
                ry,
                x,
                y
            );
        }
    }

    /// 测试 4x4 Hilbert 曲线的往返转换
    ///
    /// 验证所有坐标的 xy2d -> d2xy 往返转换都能正确还原
    #[test]
    fn test_hilbert_4x4() {
        for y in 0..4 {
            for x in 0..4 {
                let d = xy2d(4, x, y);
                let (rx, ry) = d2xy(4, d);
                assert_eq!(
                    (rx, ry),
                    (x, y),
                    "Roundtrip failed for 4x4 at x={}, y={}",
                    x,
                    y
                );
            }
        }
    }

    /// 测试多种尺寸 Hilbert 曲线的往返转换
    ///
    /// 覆盖 2x2, 4x4, 8x8, 16x16 四种尺寸，确保算法的通用性
    #[test]
    fn test_roundtrip() {
        for side in [2, 4, 8, 16] {
            for y in 0..side {
                for x in 0..side {
                    let d = xy2d(side, x, y);
                    let (rx, ry) = d2xy(side, d);
                    assert_eq!(
                        (rx, ry),
                        (x, y),
                        "Roundtrip failed for side={}, x={}, y={}",
                        side,
                        x,
                        y
                    );
                }
            }
        }
    }

    /// 测试 Hilbert 索引的基本功能
    ///
    /// 验证边界情况和初始位置的正确性
    #[test]
    fn test_hilbert_inverse() {
        assert_eq!(hilbert_index(0, 0, 2), 0);
        assert_eq!(hilbert_index(1, 0, 2), 1);

        let (x, y) = hilbert_inverse(0, 2);
        assert_eq!((x, y), (0, 0));
    }
}
