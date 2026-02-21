# Hilbert Image Obfuscator

基于 Hilbert 曲线和伪随机数生成器的图像混淆与解混淆工具。

## 项目简介

本项目实现了一种图像混淆技术，通过 Fisher-Yates 洗牌算法打乱图像像素顺序，达到图像隐藏/保护的目的。

### 核心特性

- **确定性混淆**：使用种子（seed）控制混淆结果，相同种子可完全恢复原始图像
- **种子敏感性**：不同种子产生完全不同的混淆结果
- **简单高效**：时间复杂度 O(n)，空间复杂度 O(n)
- **图形界面**：提供友好的桌面 GUI 操作界面
- **跨平台支持**：支持 Windows、macOS、Linux

## 算法原理

### 混淆过程

1. **像素展平**：将二维图像的所有像素提取为一维数组
2. **生成排列**：使用 SplitMix64 伪随机数生成器生成确定性的索引排列
3. **像素重排**：根据排列重新组织像素位置
4. **图像重建**：将一维像素向量重新转换为二维图像

### 关键组件

- **Hilbert 曲线**：空间填充曲线，用于2D坐标到1D索引的映射（本版本中可选使用）
- **SplitMix64**：高质量的64位伪随机数生成器，确保随机性的同时保持确定性
- **Fisher-Yates 洗牌**：经典的洗牌算法，确保每个排列概率相等

## 项目结构

```
混淆解混淆图片/
├── src/
│   ├── lib.rs          # 库入口，公共 API 导出
│   ├── main.rs         # 图形界面应用程序
│   ├── hilbert.rs     # Hilbert 曲线实现
│   ├── image_ops.rs   # 图像操作（混淆/解混淆）
│   └── prng.rs        # 伪随机数生成器
├── tests/
│   └── integration_test.rs  # 集成测试
├── Cargo.toml         # 项目配置
└── README.md          # 项目文档
```

## 构建与运行

### 前置要求

- Rust 1.56 或更高版本
- 支持的操作系统：Windows、macOS、Linux

### 编译项目

```bash
# 调试模式编译
cargo build

# 发布模式编译
cargo build --release
```

### 运行应用程序

```bash
# 运行 GUI 应用程序
cargo run
```

### 运行测试

```bash
# 运行所有测试
cargo test
```

## 使用指南

### 图形界面操作

1. **打开原始图片**：点击"打开原始图片"按钮，选择要混淆的图像文件
2. **设置种子**：在种子输入框中输入一个数值（任意正整数）
3. **执行混淆**：点击"混淆"按钮，图像将被混淆
4. **执行解混淆**：点击"解混淆"按钮，使用相同种子可以恢复原始图像
5. **保存结果**：可以使用"保存混淆后图片"或"保存解混淆图片"按钮保存结果

### 命令行使用（库 API）

```rust
use hilbert_image_obfuscator::{obfuscate, deobfuscate};
use image::RgbaImage;

fn main() {
    // 加载图像
    let img = image::open("input.png").unwrap().to_rgba8();
    
    // 设置种子
    let seed = 12345;
    
    // 混淆图像
    let (obfuscated, side) = obfuscate(&img, seed);
    obfuscated.save("obfuscated.png").unwrap();
    
    // 解混淆图像
    let (deobfuscated, _) = deobfuscate(&obfuscated, seed, img.width(), img.height(), side);
    deobfuscated.save("deobfuscated.png").unwrap();
}
```

## 依赖说明

| 依赖 | 版本 | 说明 |
|------|------|------|
| eframe | 0.23 | egui 框架，用于构建 GUI |
| egui | 0.23 | 图形界面库 |
| image | 0.24 | 图像处理库 |
| rfd | 0.11 | 原生文件对话框 |
| hilbert_2d | 1.1 | Hilbert 曲线算法 |

## 安全性说明

⚠️ **重要提示**：

- 本项目实现的是图像混淆技术，**不是加密算法**
- 仅适用于图像隐藏、版权保护、教学演示等场景
- 不适用于安全敏感的数据保护场景
- 如需高安全性保护，请结合专业加密算法使用

## 常见问题

### Q: 为什么需要种子？

A: 种子用于初始化伪随机数生成器。相同的种子产生相同的随机序列，确保混淆和解混淆的可逆性。

### Q: 混淆后的图像看起来是什么样的？

A: 图像会被完全打乱，看起来像随机噪声。不同种子产生不同的噪声模式。

### Q: 如何知道解混淆是否成功？

A: 使用相同种子解混淆后，图像应该与原始图像完全一致。可以通过比较图像尺寸和像素值来验证。

## 许可证

MIT License

## 贡献指南

欢迎提交 Issue 和 Pull Request！

## 联系方式

如有问题，请提交 Issue。
