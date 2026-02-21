//! Hilbert Image Obfuscator - 图像混淆与解混淆库
//!
//! 这是一个使用 Hilbert 曲线和伪随机数生成器实现图像像素打乱的工具库。
//! 提供命令行和图形界面两种使用方式。
//!
//! ## 项目概述
//!
//! 本项目实现了一种图像混淆技术，通过打乱像素顺序来达到图像隐藏/保护的目的。
//! 与传统加密不同，这种方法：
//! - 不改变像素值，只改变位置
//! - 使用种子（seed）控制混淆结果
//! - 相同的种子可以完全恢复原始图像
//! - 算法简单高效
//!
//! ## 核心算法
//!
//! 1. **像素展平**：将二维图像的所有像素提取为一维数组
//! 2. **Hilbert 曲线映射**（可选）：将 2D 坐标映射到 1D Hilbert 曲线索引
//! 3. **Fisher-Yates 洗牌**：使用 SplitMix64 伪随机数生成器生成确定性随机排列
//! 4. **像素重排**：按照随机排列重新组织像素
//! 5. **图像重建**：将一维像素数组转换回二维图像
//!
//! ## 主要功能模块
//!
//! - `hilbert`: Hilbert 曲线坐标转换
//! - `image_ops`: 图像混淆/解混淆操作
//! - `prng`: SplitMix64 伪随机数生成器

pub mod hilbert;
pub mod image_ops;
pub mod prng;

// 公开导出主要 API
pub use hilbert::{hilbert_index, hilbert_inverse};
pub use image_ops::{deobfuscate, next_power_of_two, obfuscate, save_image};
pub use prng::SplitMix64;
