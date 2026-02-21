# 新手部署指南

本指南将一步一步教你如何从零开始搭建开发环境并编译运行本项目。

---

## 目录

1. [安装 Git](#1-安装-git)
2. [安装 Rust](#2-安装-rust)
3. [克隆项目](#3-克隆项目)
4. [编译项目](#4-编译项目)
5. [运行程序](#5-运行程序)
6. [找到生成的 exe 文件](#6-找到生成的-exe-文件)

---

## 1. 安装 Git

Git 是一个版本管理工具，用来下载和管理项目代码。

### Windows 用户

1. 访问 Git 官网：https://git-scm.com/download/win
2. 下载安装包（一般会自动下载 64-bit 版本）
3. 运行安装包，全部选择默认选项即可
4. 安装完成后，在开始菜单搜索 "Git Bash" 并打开

### 验证安装

打开 CMD 或 PowerShell，输入以下命令：

```bash
git --version
```

如果显示版本号（如 `git version 2.x.x`），说明安装成功。

---

## 2. 安装 Rust

Rust 是本项目的编程语言，需要先安装 Rust 工具链。

### Windows 用户

1. 打开 https://rustup.rs
2. 点击 "Download Rustup-init.exe"
3. 运行下载的文件
4. 安装时选择 "1) Proceed with installation (default)"
5. 等待安装完成

### 验证安装

打开一个新的 CMD 或 PowerShell 窗口（重要：需要重新打开终端），输入：

```bash
rustc --version
cargo --version
```

如果显示版本号，说明安装成功。

---

## 3. 克隆项目

将项目代码下载到本地。

### 方法一：使用 Git Bash

```bash
git clone https://github.com/你的GitHub用户名/混淆解混淆图片.git
cd 混淆解混淆图片
```

### 方法二：手动下载

1. 访问你的 GitHub 仓库页面
2. 点击绿色 "Code" 按钮
3. 点击 "Download ZIP"
4. 解压下载的 ZIP 文件
5. 在解压后的文件夹空白处右键，选择 "在终端中打开"

---

## 4. 编译项目

### 调试模式（编译较快，适合开发）

```bash
cargo build
```

### 发布模式（编译较慢，但生成的 exe 体积更小、运行更快）

```bash
cargo build --release
```

**第一次编译时**，Cargo 会自动下载依赖库，需要等待几分钟。后续编译会快很多。

---

## 5. 运行程序

编译完成后，运行程序：

```bash
cargo run
```

或者如果你想运行发布版本：

```bash
cargo run --release
```

程序启动后会显示一个图形界面窗口，你可以通过界面对图片进行混淆和解混淆操作。

---

## 6. 找到生成的 exe 文件

### 调试模式

```
项目根目录\target\debug\hilbert-image-obfuscator.exe
```

### 发布模式

```
项目根目录\target\release\hilbert-image-obfuscator.exe
```

**使用方法：**

1. 打开文件资源管理器
2. 进入项目根目录
3. 进入 `target` 文件夹
4. 进入 `release` 文件夹（发布模式）或 `debug` 文件夹（调试模式）
5. 找到 `hilbert-image-obfuscator.exe`，双击即可运行

---

## 常见问题

### Q: 编译时出现错误提示

A: 确保已正确安装 Rust 并重启了终端。如果问题仍然存在，尝试运行以下命令更新 Rust：

```bash
rustup update
```

### Q: 找不到 exe 文件

A: 确保已执行过 `cargo build` 或 `cargo build --release` 命令。首次编译可能需要几分钟时间。

### Q: 双击 exe 闪退

A: 可能是缺少运行库。尝试通过 `cargo run` 运行，终端会显示错误信息。

---

## 使用图形界面

程序界面包含以下功能：

| 功能 | 说明 |
|------|------|
| 打开原始图片 | 选择要处理的图片文件 |
| 设置种子 | 输入任意正整数（如 12345）|
| 混淆 | 将图片打乱 |
| 解混淆 | 使用相同种子恢复图片 |
| 保存混淆后图片 | 导出混淆结果 |
| 保存解混淆图片 | 导出恢复后的图片 |

**重要：** 记住你使用的种子！使用相同的种子才能正确解混淆。

---

## 推送到 GitHub 并自动发布

如果你想每次推送代码时自动编译并生成 exe：

### 步骤 1：创建 GitHub 仓库

1. 登录 GitHub
2. 点击 "+" -> "New repository"
3. 输入仓库名（如 `hilbert-image-obfuscator`）
4. 点击 "Create repository"

### 步骤 2：推送代码

```bash
git remote add origin https://github.com/你的用户名/仓库名.git
git branch -M main
git push -u origin main
```

### 步骤 3：创建版本标签

```bash
git tag v0.1.0
git push origin v0.1.0
```

推送标签后，GitHub Actions 会自动：
1. 在 Windows 环境编译项目
2. 创建 GitHub Release
3. 上传 exe 文件

你可以在 GitHub 仓库的 "Actions" 页面查看编译进度，在 "Releases" 页面下载编译好的 exe。

---

完成！现在你已经掌握了从零开始使用本项目的全部步骤。
