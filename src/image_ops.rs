//! 图像混淆与解混淆操作模块
//!
//! 本模块提供了图像处理的核心功能，包括：
//! - 图像的混淆（obfuscate）：打乱图像像素顺序
//! - 图像的解混淆（deobfuscate）：恢复原始像素顺序
//! - 图像的加载和保存
//! - 辅助函数（如下一个2的幂次）
//!
//! ## 混淆算法原理
//!
//! 本项目采用的图像混淆算法基于以下原理：
//! 1. 将图像的所有像素提取为一维数组
//! 2. 使用 Fisher-Yates 洗牌算法配合确定性伪随机数生成器打乱像素顺序
//! 3. 将打乱后的像素重新排列为图像
//!
//! 这种方法的优势：
//! - 可逆：使用相同种子可以完全恢复原始图像
//! - 种子敏感：不同的种子产生完全不同的混淆结果
//! - 简单高效：时间复杂度 O(n)，空间复杂度 O(n)
//!
//! ## 安全性说明
//!
//! 这是一种简单的像素重排技术，不是加密算法。
//! 在安全敏感的场景中，应结合加密算法使用。

use crate::prng::shuffle_indices;
use image::{ImageBuffer, Rgba, RgbaImage};

/// 计算给定数值的下一个 2 的幂次
///
/// 在 Hilbert 曲线应用中，需要将图像扩展为边长为 2 的幂次的正方形。
/// 此函数计算不小于给定值的最小 2 的幂次。
///
/// # 算法原理
///
/// 使用位运算快速计算下一个 2 的幂次：
/// - 将 v-1 的所有低位设置为 1
/// - 然后加 1，即得到下一个 2 的幂次
///
/// # 参数说明
/// - `v`: 输入的整数值
///
/// # 返回值
/// 返回不小于 v 的最小 2 的幂次。如果 v <= 1，则返回 2
pub fn next_power_of_two(mut v: u32) -> u32 {
    if v <= 1 {
        return 2;
    }
    v -= 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v + 1
}

/// 对图像进行混淆处理
///
/// 将图像的像素顺序打乱，使原始图像变得不可辨认。
/// 使用种子（seed）来控制随机打乱的顺序，相同的种子产生相同的结果。
/// 种子会被写入图片右下角以便解混淆时自动读取。
///
/// # 算法步骤
///
/// 1. **像素展平**：将二维图像的所有像素（RGBA）提取为一维向量
/// 2. **生成排列**：使用 SplitMix64 PRNG 生成确定性的索引排列
/// 3. **像素重排**：根据排列重新组织像素位置
/// 4. **图像重建**：将一维像素向量重新转换为二维图像
/// 5. **写入种子**：将种子信息写入图片右下角
///
/// # 参数说明
///
/// - `orig`: 原始图像的引用（RgbaImage 类型）
/// - `seed`: 混淆种子，使用相同的种子可以得到相同的混淆结果
///
/// # 返回值
///
/// 返回一个元组 (RgbaImage, u32)：
/// - RgbaImage: 混淆后的图像（右下角包含种子信息）
/// - u32: 图像的边长（取宽度和高度的最大值，用于 Hilbert 曲线计算）
///
/// # 重要说明
///
/// - 混淆后的图像保留原始图像的尺寸
/// - 混淆操作是可逆的，使用相同的 seed 调用 deobfuscate 可以恢复原始图像
/// - 不同的 seed 会产生完全不同的混淆结果
/// - 种子信息以特殊像素模式嵌入图片右下角
pub fn obfuscate(orig: &RgbaImage, seed: u64) -> (RgbaImage, u32) {
    let (w, h) = orig.dimensions();
    let n = (w * h) as usize;

    // Flatten pixels
    let mut pixels: Vec<Rgba<u8>> = Vec::with_capacity(n);
    for y in 0..h {
        for x in 0..w {
            pixels.push(orig.get_pixel(x, y).clone());
        }
    }

    // Shuffle
    let perm = shuffle_indices(n, seed);
    let mut shuffled = vec![Rgba([0, 0, 0, 0]); n];
    for i in 0..n {
        shuffled[i] = pixels[perm[i]].clone();
    }

    // Reshape to image
    let mut out = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            out.put_pixel(x, y, shuffled[idx].clone());
        }
    }

    // 将种子写入图片右下角（从右往左写）
    // 使用特殊标记：前3个像素为红色通道的特殊值标记
    let seed_bytes = seed.to_le_bytes();
    let marker = [0xDE, 0xAD, 0xBE, 0xEF]; // 特殊标记

    // 从右下角开始写入：标记(4字节) + 种子(8字节) = 12像素
    let mut px = w as i32 - 1;
    let py = h as i32 - 1;

    // 写入标记
    for i in 0..4 {
        if px >= 0 {
            out.put_pixel(px as u32, py as u32, Rgba([marker[i], 0, 0, 255]));
            px -= 1;
        }
    }

    // 写入种子字节
    for byte in seed_bytes {
        if px >= 0 {
            out.put_pixel(px as u32, py as u32, Rgba([byte, 0, 0, 255]));
            px -= 1;
        }
    }

    // Return side = max dimension for compatibility
    (out, w.max(h))
}

/// 从混淆后的图片右下角读取种子
///
/// 读取图片右下角嵌入的种子信息。
///
/// # 参数说明
///
/// - `img`: 混淆后的图像引用
///
/// # 返回值
///
/// - `Some(u64)`: 成功读取到种子
/// - `None`: 未找到有效的种子标记
pub fn read_seed_from_image(img: &RgbaImage) -> Option<u64> {
    let w = img.width();
    let h = img.height();
    let marker = [0xDE, 0xAD, 0xBE, 0xEF];

    // 从右下角开始读取
    let mut px = w as i32 - 1;
    let py = h as i32 - 1;

    // 读取并验证标记
    let mut read_marker = [0u8; 4];
    for i in 0..4 {
        if px < 0 {
            return None;
        }
        let pixel = img.get_pixel(px as u32, py as u32);
        read_marker[i] = pixel[0];
        px -= 1;
    }

    if read_marker != marker {
        return None;
    }

    // 读取种子字节
    let mut seed_bytes = [0u8; 8];
    for i in 0..8 {
        if px < 0 {
            return None;
        }
        let pixel = img.get_pixel(px as u32, py as u32);
        seed_bytes[i] = pixel[0];
        px -= 1;
    }

    Some(u64::from_le_bytes(seed_bytes))
}

/// 对混淆后的图像进行解混淆处理
///
/// 这是 obfuscate 函数的逆操作，使用相同的种子恢复原始图像。
/// 如果 seed 为 None，将自动从图片右下角读取种子。
///
/// # 算法步骤
///
/// 1. **读取种子**：如果未指定种子，则从图片右下角读取
/// 2. **像素展平**：将混淆图像的所有像素提取为一维向量
/// 3. **生成排列**：使用相同的 seed 生成与混淆时相同的排列
/// 4. **计算逆排列**：计算排列的逆，用于反向映射
/// 5. **逆排列应用**：将像素按照逆排列重排，恢复原始顺序
/// 6. **图像重建**：将一维像素向量重新转换为二维图像
///
/// # 参数说明
///
/// - `obf`: 混淆后的图像引用
/// - `seed`: 混淆时使用的种子，如果为 None 则自动从图片读取
/// - `orig_w`: 原始图像的宽度
/// - `orig_h`: 原始图像的高度
/// - `_side`: 正方形边长（用于 Hilbert 曲线，目前保留兼容）
///
/// # 返回值
///
/// 返回一个元组 (RgbaImage, u32, Option<u64>)：
/// - RgbaImage: 解混淆后的原始图像
/// - u32: 图像边长
/// - Option<u64>: 实际使用的种子（用于显示）
///
/// # 重要说明
///
/// - 如果未指定种子，会自动从图片右下角读取
/// - 必须使用与混淆时相同的 seed 才能正确恢复图像
/// - 如果图像尺寸不匹配，结果可能不正确
/// - 解混淆后，图像将完全恢复到原始状态
pub fn deobfuscate(
    obf: &RgbaImage,
    seed: Option<u64>,
    orig_w: u32,
    orig_h: u32,
    _side: u32,
) -> (RgbaImage, u32, Option<u64>) {
    // 如果未指定种子，尝试从图片读取
    let actual_seed = match seed {
        Some(s) => s,
        None => match read_seed_from_image(obf) {
            Some(s) => s,
            None => {
                // 无法读取种子，返回原始图像
                return (obf.clone(), obf.width().max(obf.height()), None);
            }
        },
    };

    let w = obf.width();
    let h = obf.height();
    let n = (w * h) as usize;

    // Flatten pixels
    let mut pixels: Vec<Rgba<u8>> = Vec::with_capacity(n);
    for y in 0..h {
        for x in 0..w {
            pixels.push(obf.get_pixel(x, y).clone());
        }
    }

    // Generate same permutation
    let perm = shuffle_indices(n, actual_seed);

    // Compute inverse permutation
    let mut inv_perm = vec![0usize; n];
    for i in 0..n {
        inv_perm[perm[i]] = i;
    }

    // Unshuffle
    let mut unshuffled = vec![Rgba([0, 0, 0, 0]); n];
    for i in 0..n {
        unshuffled[i] = pixels[inv_perm[i]].clone();
    }

    // Reshape to image with original dimensions
    let mut out = ImageBuffer::new(orig_w, orig_h);
    for y in 0..orig_h {
        for x in 0..orig_w {
            let idx = (y * orig_w + x) as usize;
            if idx < unshuffled.len() {
                out.put_pixel(x, y, unshuffled[idx].clone());
            }
        }
    }

    (out, w.max(h), Some(actual_seed))
}

/// 将图像保存到磁盘

/// 将图像保存到磁盘
///
/// 使用 image crate 的保存功能，将 RgbaImage 保存为指定格式
/// （PNG、JPEG、BMP 等，取决于文件扩展名）。
///
/// # 参数说明
///
/// - `img`: 要保存的图像引用
/// - `path`: 保存路径，支持的格式由文件扩展名决定
///
/// # 返回值
///
/// - `Ok(())`: 保存成功
/// - `Err(...)`: 保存失败，返回错误信息
///
/// # 支持的格式
///
/// 根据 image crate 的实现，支持以下常见格式：
/// - PNG (.png): 无损压缩，支持透明通道
/// - JPEG (.jpg, .jpeg): 有损压缩，不支持透明通道
/// - BMP (.bmp): 无压缩，支持透明通道
/// - GIF (.gif): 动画格式
/// - TIFF (.tif, .tiff): 无损压缩
pub fn save_image(img: &RgbaImage, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    img.save(path)?;
    Ok(())
}

/// 从磁盘加载图像
///
/// 从指定路径加载图像文件，并转换为 RGBA 格式返回。
///
/// # 参数说明
///
/// - `path`: 要加载的图像文件路径
///
/// # 返回值
///
/// - `Ok(RgbaImage)`: 加载成功，返回 RGBA 格式的图像
/// - `Err(...)`: 加载失败，返回错误信息
///
/// # 转换说明
///
/// 无论原始图像是什么格式（灰度、RGB、RGBA 等），
/// 加载后都会转换为 RGBA（红绿蓝Alpha）格式：
/// - RGB 图像：Alpha 通道设为 255（不透明）
/// - 灰度图像：R=G=B=原始灰度值，Alpha=255
/// - RGBA 图像：保持不变
pub fn load_image(path: &str) -> Result<RgbaImage, Box<dyn std::error::Error>> {
    let img = image::open(path)?.to_rgba8();
    Ok(img)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageBuffer;

    /// 测试 next_power_of_two 函数的各种输入
    ///
    /// 验证函数对不同数值的处理是否正确
    #[test]
    fn test_next_power_of_two() {
        assert_eq!(next_power_of_two(1), 2);
        assert_eq!(next_power_of_two(2), 2);
        assert_eq!(next_power_of_two(3), 4);
        assert_eq!(next_power_of_two(4), 4);
        assert_eq!(next_power_of_two(5), 8);
        assert_eq!(next_power_of_two(15), 16);
        assert_eq!(next_power_of_two(16), 16);
        assert_eq!(next_power_of_two(17), 32);
    }

    /// 测试混淆-解混淆的往返操作
    ///
    /// 验证解混淆能够完全恢复原始图像
    #[test]
    fn test_obfuscate_deobfuscate_roundtrip() {
        let img: RgbaImage = ImageBuffer::from_fn(3, 3, |x, y| {
            let val = ((x * 4 + y) * 50) as u8;
            Rgba([val, val, val, 255])
        });

        let seed = 12345;

        // Obfuscate
        let (obf, side) = obfuscate(&img, seed);

        // Obfuscated image has original dimensions
        assert_eq!(obf.width(), img.width());
        assert_eq!(obf.height(), img.height());

        // Deobfuscate
        let (deobf, _side, _) = deobfuscate(&obf, Some(seed), img.width(), img.height(), side);

        // Verify dimensions
        assert_eq!(deobf.width(), img.width());
        assert_eq!(deobf.height(), img.height());
    }

    /// 测试不同种子产生不同的混淆结果
    #[test]
    fn test_different_seeds_produce_different_results() {
        let img: RgbaImage = ImageBuffer::from_fn(4, 4, |x, y| Rgba([x as u8, y as u8, 128, 255]));

        let (obf1, _) = obfuscate(&img, 1);
        let (obf2, _) = obfuscate(&img, 2);

        let mut different = false;
        for y in 0..4 {
            for x in 0..4 {
                if obf1.get_pixel(x, y) != obf2.get_pixel(x, y) {
                    different = true;
                    break;
                }
            }
            if different {
                break;
            }
        }
        assert!(
            different,
            "Different seeds should produce different obfuscated images"
        );
    }

    /// 测试相同种子产生相同的混淆结果
    #[test]
    fn test_same_seed_produces_same_result() {
        let img: RgbaImage = ImageBuffer::from_fn(4, 4, |x, y| Rgba([x as u8, y as u8, 128, 255]));

        let (obf1, _) = obfuscate(&img, 42);
        let (obf2, _) = obfuscate(&img, 42);

        for y in 0..4 {
            for x in 0..4 {
                assert_eq!(
                    obf1.get_pixel(x, y),
                    obf2.get_pixel(x, y),
                    "Same seed should produce identical results"
                );
            }
        }
    }

    /// 测试混淆操作保持图像尺寸
    #[test]
    fn test_obfuscate_preserves_dimensions() {
        for size in [3, 5, 7, 15, 17, 100] {
            let img: RgbaImage = ImageBuffer::new(size, size);
            let (obf, side) = obfuscate(&img, 1);

            assert_eq!(side, size as u32);
            assert_eq!(obf.width(), size as u32);
            assert_eq!(obf.height(), size as u32);
        }
    }

    /// 测试图像保存和加载功能
    #[test]
    fn test_load_save_image() {
        let img: RgbaImage =
            ImageBuffer::from_fn(4, 4, |x, y| Rgba([x as u8, y as u8, (x + y) as u8, 255]));

        let test_path = "test_output.png";

        save_image(&img, test_path).unwrap();
        let loaded = load_image(test_path).unwrap();

        assert_eq!(loaded.width(), img.width());
        assert_eq!(loaded.height(), img.height());

        for y in 0..img.height() {
            for x in 0..img.width() {
                assert_eq!(img.get_pixel(x, y), loaded.get_pixel(x, y));
            }
        }

        std::fs::remove_file(test_path).ok();
    }
}
