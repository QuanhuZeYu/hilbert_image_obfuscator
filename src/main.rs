//! Hilbert Image Obfuscator - 图形界面应用程序
//!
//! 本文件实现了基于 egui 的桌面应用程序，提供图像混淆和解混淆的图形化操作界面。
//!
//! ## 功能说明
//!
//! - 打开原始图像进行混淆处理
//! - 打开已混淆的图像进行解混淆
//! - 可调节的种子值用于控制混淆结果
//! - 实时预览原始、混淆后、解混淆后的图像
//! - 支持图像的保存操作
//!
//! ## 界面布局
//!
//! - 左侧面板：操作控制区（打开图片、设置参数、执行操作、保存结果）
//! - 中央区域：图像预览区（可缩放查看三张图像）
//! - 顶部：应用标题

use hilbert_image_obfuscator::{
    deobfuscate, next_power_of_two, obfuscate, read_seed_from_image, save_image,
};
use image::RgbaImage;
use std::cell::RefCell;
use std::collections::HashMap;

/// 跟踪缩放焦点的图像类型
#[derive(Clone, Copy, PartialEq, Eq)]
enum FocusedImage {
    Orig,
    Obfuscated,
    Deobfuscated,
    None,
}

/// 应用程序状态结构体
///
/// 存储应用程序运行过程中的所有状态数据，包括：
/// - 原始图像、混淆图像、解混淆图像
/// - 图像尺寸参数
/// - 种子值
/// - 缩放级别
/// - 纹理缓存（用于图像显示优化）
struct AppState {
    /// 原始输入图像（加载但未处理）
    orig: Option<RgbaImage>,
    /// 混淆后的图像
    obf: Option<RgbaImage>,
    /// 解混淆后的图像
    deobf: Option<RgbaImage>,
    /// 原始图像宽度
    orig_w: u32,
    /// 原始图像高度
    orig_h: u32,
    /// 正方形边长（用于 Hilbert 曲线）
    side: u32,
    /// 混淆/解混淆使用的种子值
    seed: u64,
    /// 原始图像预览缩放比例
    orig_zoom: f32,
    /// 混淆图像预览缩放比例
    obf_zoom: f32,
    /// 解混淆图像预览缩放比例
    deobf_zoom: f32,
    /// 当前聚焦的图像（用于滚轮缩放）
    focused_zoom: FocusedImage,
    /// 是否手动指定解混淆种子
    use_manual_seed: bool,
    /// 从图片中读取的种子
    read_seed: Option<u64>,
    /// egui 纹理缓存，用于加速图像显示
    texture_cache: RefCell<HashMap<String, (u32, u32, egui::ColorImage)>>,
}

/// 为 AppState 实现 Default trait，提供默认初始状态
impl Default for AppState {
    fn default() -> Self {
        AppState {
            orig: None,
            obf: None,
            deobf: None,
            orig_w: 0,
            orig_h: 0,
            side: 2,
            seed: 1234567890,
            orig_zoom: 1.0,
            obf_zoom: 1.0,
            deobf_zoom: 1.0,
            focused_zoom: FocusedImage::None,
            use_manual_seed: false,
            read_seed: None,
            texture_cache: RefCell::new(HashMap::new()),
        }
    }
}

/// 为 AppState 实现 Clone trait
impl Clone for AppState {
    fn clone(&self) -> Self {
        AppState {
            orig: self.orig.clone(),
            obf: self.obf.clone(),
            deobf: self.deobf.clone(),
            orig_w: self.orig_w,
            orig_h: self.orig_h,
            side: self.side,
            seed: self.seed,
            orig_zoom: self.orig_zoom,
            obf_zoom: self.obf_zoom,
            deobf_zoom: self.deobf_zoom,
            focused_zoom: self.focused_zoom,
            use_manual_seed: self.use_manual_seed,
            read_seed: self.read_seed,
            texture_cache: RefCell::new(HashMap::new()),
        }
    }
}

/// 配置字体设置，添加中文字体支持
///
/// 在 Windows 系统上，尝试加载系统自带的中文字体：
/// - 微软雅黑 (msyh.ttc)
/// - 宋体 (simsun.ttc)
/// - 黑体 (simhei.ttf)
///
/// 如果找到可用的中文字体，将其添加到 egui 的字体配置中，
/// 以确保界面能够正确显示中文。
///
/// # 返回值
/// 返回配置好的 egui::FontDefinitions
fn font_config() -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();

    #[cfg(target_os = "windows")]
    {
        let font_paths = [
            "C:\\Windows\\Fonts\\msyh.ttc",
            "C:\\Windows\\Fonts\\simsun.ttc",
            "C:\\Windows\\Fonts\\simhei.ttf",
        ];

        for font_path in font_paths {
            if std::path::Path::new(font_path).exists() {
                if let Ok(font_data) = std::fs::read(font_path) {
                    fonts.font_data.insert(
                        "chinese".to_owned(),
                        egui::FontData::from_owned(font_data).into(),
                    );

                    fonts
                        .families
                        .entry(egui::FontFamily::Proportional)
                        .or_insert_with(Vec::new)
                        .insert(0, "chinese".to_owned());

                    fonts
                        .families
                        .entry(egui::FontFamily::Monospace)
                        .or_insert_with(Vec::new)
                        .insert(0, "chinese".to_owned());

                    break;
                }
            }
        }
    }

    fonts
}

/// 应用程序入口点
///
/// 使用 eframe 框架启动原生桌面应用程序。
/// 配置字体并初始化应用程序状态。
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Hilbert Image Obfuscator",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_fonts(font_config());
            Ok(Box::new(AppState::default()))
        }),
    )
    .expect("Failed to run native application");
}

/// 为 AppState 实现 eframe::App trait，处理 GUI 更新逻辑
///
/// 这是 egui 应用程序的核心更新函数，每帧都会被调用。
/// 负责构建整个用户界面和处理用户交互。
impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        use egui::{CentralPanel, SidePanel, TopBottomPanel};

        // 处理 Ctrl+滚轮 缩放
        if ctx.input(|i| i.modifiers.ctrl) {
            let scroll_delta = ctx.input(|i| {
                i.events.iter().find_map(|e| {
                    if let egui::Event::MouseWheel { delta, .. } = e {
                        Some(delta.y)
                    } else {
                        None
                    }
                })
            });
            if let Some(delta) = scroll_delta {
                let zoom_delta = if delta > 0.0 { 0.1 } else { -0.1 };
                match self.focused_zoom {
                    FocusedImage::Orig => {
                        self.orig_zoom = (self.orig_zoom + zoom_delta).clamp(0.1, 5.0);
                    }
                    FocusedImage::Obfuscated => {
                        self.obf_zoom = (self.obf_zoom + zoom_delta).clamp(0.1, 5.0);
                    }
                    FocusedImage::Deobfuscated => {
                        self.deobf_zoom = (self.deobf_zoom + zoom_delta).clamp(0.1, 5.0);
                    }
                    FocusedImage::None => {}
                }
                self.texture_cache.borrow_mut().clear();
            }
        }

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.label("Hilbert 曲线 + SplitMix64 的混淆算法");
        });

        SidePanel::left("left").show(ctx, |ui| {
            ui.heading("输入");
            if ui.button("打开原始图片").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "bmp"])
                    .pick_file()
                {
                    if let Ok(img) = image::open(&path) {
                        let rgba = img.to_rgba8();
                        self.orig_w = rgba.width();
                        self.orig_h = rgba.height();
                        self.side = next_power_of_two(self.orig_w.max(self.orig_h)).max(2);
                        self.orig = Some(rgba);
                        self.obf = None;
                        self.deobf = None;
                        self.texture_cache.borrow_mut().clear();
                    }
                }
            }
            if ui.button("打开混淆图片").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "bmp"])
                    .pick_file()
                {
                    if let Ok(img) = image::open(&path) {
                        let rgba = img.to_rgba8();
                        let w = rgba.width();
                        let h = rgba.height();
                        let side = w.max(h);
                        // 先读取种子（需要借用rgba）
                        self.read_seed = read_seed_from_image(&rgba);
                        if let Some(rs) = self.read_seed {
                            self.seed = rs;
                        }
                        // 再保存图像
                        self.obf = Some(rgba);
                        self.side = side;
                        self.orig_w = w;
                        self.orig_h = h;
                        self.deobf = None;
                        self.texture_cache.borrow_mut().clear();
                    }
                }
            }
            ui.separator();
            ui.label("种子 (用于混淆的伪随机序列)");
            let mut seed_text = self.seed.to_string();
            ui.text_edit_singleline(&mut seed_text);
            if let Ok(v) = seed_text.parse::<u64>() {
                self.seed = v;
            }
            ui.separator();
            ui.checkbox(&mut self.use_manual_seed, "手动指定解混淆种子");
            if let Some(rs) = self.read_seed {
                if !self.use_manual_seed {
                    ui.label(format!("图片中的种子: {}", rs));
                }
            }
            ui.separator();
            if ui.button("混淆").clicked() {
                if let Some(ref orig) = self.orig {
                    let (out, side) = obfuscate(orig, self.seed);
                    self.obf = Some(out);
                    self.side = side;
                    self.texture_cache.borrow_mut().clear();
                }
            }
            if ui.button("解混淆").clicked() {
                if let Some(ref obf) = self.obf {
                    let seed_param = if self.use_manual_seed {
                        Some(self.seed)
                    } else {
                        None
                    };
                    let (out, _side, used_seed) =
                        deobfuscate(obf, seed_param, self.orig_w, self.orig_h, self.side);
                    self.deobf = Some(out);
                    if let Some(used) = used_seed {
                        self.read_seed = Some(used);
                    }
                    self.texture_cache.borrow_mut().clear();
                }
            }
            ui.separator();
            ui.heading("保存");
            if ui.button("保存混淆后图片").clicked() {
                if let Some(ref obf) = self.obf {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("PNG", &["png"])
                        .set_file_name("obfuscated.png")
                        .save_file()
                    {
                        let path_str = path.to_string_lossy().to_string();
                        if let Err(e) = save_image(obf, &path_str) {
                            eprintln!("保存失败: {}", e);
                        }
                    }
                }
            }
            if ui.button("保存解混淆图片").clicked() {
                if let Some(ref deobf) = self.deobf {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("PNG", &["png"])
                        .set_file_name("deobfuscated.png")
                        .save_file()
                    {
                        let path_str = path.to_string_lossy().to_string();
                        if let Err(e) = save_image(deobf, &path_str) {
                            eprintln!("保存失败: {}", e);
                        }
                    }
                }
            }
        });

        CentralPanel::default().show(ctx, |ui| {
            use egui::ScrollArea;
            ui.heading("结果预览\n使用滑条或 Ctrl+滚轮 缩放");
            ScrollArea::vertical().show(ui, |ui| {
                ui.vertical(|ui| {
                    if let Some(ref img) = self.orig {
                        let is_focused = self.focused_zoom == FocusedImage::Orig;
                        let mut focused = is_focused;
                        ui.horizontal(|ui| {
                            ui.label(format!("原始图像 {}x{}", img.width(), img.height()));
                            ui.separator();
                            if ui.button("-").clicked() {
                                self.orig_zoom = (self.orig_zoom - 0.1).max(0.1);
                                self.texture_cache.borrow_mut().clear();
                            }
                            ui.label(format!("{:.0}%", self.orig_zoom * 100.0));
                            if ui.button("+").clicked() {
                                self.orig_zoom = (self.orig_zoom + 0.1).min(5.0);
                                self.texture_cache.borrow_mut().clear();
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.add(egui::Slider::new(&mut self.orig_zoom, 0.1..=5.0).text("缩放"));
                            if ui.checkbox(&mut focused, "聚焦").changed() {
                                self.focused_zoom = if focused {
                                    FocusedImage::Orig
                                } else {
                                    FocusedImage::None
                                };
                            }
                        });
                        ui.add_space(5.0);
                        show_image(ui, ctx, img, self.orig_zoom, "orig", &self.texture_cache);
                    } else {
                        ui.label("请先打开图片");
                    }
                    ui.separator();
                    if let Some(ref img) = self.obf {
                        let is_focused = self.focused_zoom == FocusedImage::Obfuscated;
                        let mut focused = is_focused;
                        ui.horizontal(|ui| {
                            ui.label(format!("混淆后 {}x{}", img.width(), img.height()));
                            ui.separator();
                            if ui.button("-").clicked() {
                                self.obf_zoom = (self.obf_zoom - 0.1).max(0.1);
                                self.texture_cache.borrow_mut().clear();
                            }
                            ui.label(format!("{:.0}%", self.obf_zoom * 100.0));
                            if ui.button("+").clicked() {
                                self.obf_zoom = (self.obf_zoom + 0.1).min(5.0);
                                self.texture_cache.borrow_mut().clear();
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.add(egui::Slider::new(&mut self.obf_zoom, 0.1..=5.0).text("缩放"));
                            if ui.checkbox(&mut focused, "聚焦").changed() {
                                self.focused_zoom = if focused {
                                    FocusedImage::Obfuscated
                                } else {
                                    FocusedImage::None
                                };
                            }
                        });
                        ui.add_space(5.0);
                        show_image(ui, ctx, img, self.obf_zoom, "obf", &self.texture_cache);
                    }
                    if let Some(ref img) = self.deobf {
                        let is_focused = self.focused_zoom == FocusedImage::Deobfuscated;
                        let mut focused = is_focused;
                        ui.horizontal(|ui| {
                            ui.label(format!("解混淆 {}x{}", img.width(), img.height()));
                            ui.separator();
                            if ui.button("-").clicked() {
                                self.deobf_zoom = (self.deobf_zoom - 0.1).max(0.1);
                                self.texture_cache.borrow_mut().clear();
                            }
                            ui.label(format!("{:.0}%", self.deobf_zoom * 100.0));
                            if ui.button("+").clicked() {
                                self.deobf_zoom = (self.deobf_zoom + 0.1).min(5.0);
                                self.texture_cache.borrow_mut().clear();
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.add(egui::Slider::new(&mut self.deobf_zoom, 0.1..=5.0).text("缩放"));
                            if ui.checkbox(&mut focused, "聚焦").changed() {
                                self.focused_zoom = if focused {
                                    FocusedImage::Deobfuscated
                                } else {
                                    FocusedImage::None
                                };
                            }
                        });
                        ui.add_space(5.0);
                        show_image(ui, ctx, img, self.deobf_zoom, "deobf", &self.texture_cache);
                    }
                });
            });
        });
    }
}

/// 在 egui 界面中显示图像
///
/// 将 RgbaImage 转换为 egui 可以显示的纹理，并渲染到界面上。
/// 使用缓存机制避免重复转换，提高性能。
///
/// # 参数说明
///
/// - `ui`: egui UI 上下文
/// - `ctx`: egui Context 用于加载纹理
/// - `img`: 要显示的 RGBA 图像
/// - `zoom`: 缩放比例
/// - `key`: 缓存键名，用于标识不同的图像
/// - `cache`: 纹理缓存引用，避免重复创建纹理
///
/// # 返回值
/// 返回 egui::Response，包含用户交互信息
fn show_image(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    img: &RgbaImage,
    zoom: f32,
    key: &str,
    cache: &RefCell<HashMap<String, (u32, u32, egui::ColorImage)>>,
) -> egui::Response {
    use egui::TextureOptions;
    let (w, h) = img.dimensions();

    // 检查是否需要更新缓存
    let needs_update = {
        let cache = cache.borrow();
        match cache.get(key) {
            Some((cw, ch, _)) => *cw != w || *ch != h,
            None => true,
        }
    };

    // 如果需要，更新缓存中的图像数据
    if needs_update {
        let mut colors: Vec<egui::Color32> = Vec::with_capacity((w * h) as usize);
        for p in img.pixels() {
            let c = egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]);
            colors.push(c);
        }
        let color_image = egui::ColorImage {
            size: [w as usize, h as usize],
            pixels: colors,
            source_size: egui::Vec2::new(w as f32, h as f32),
        };

        let mut cache = cache.borrow_mut();
        cache.insert(key.to_string(), (w, h, color_image));
    }

    // 从缓存获取图像数据并创建纹理
    let color_image = {
        let cache = cache.borrow();
        cache.get(key).unwrap().2.clone()
    };
    let tex = ctx.load_texture(key, color_image, TextureOptions::LINEAR);
    let scaled_size = tex.size_vec2() * zoom;

    ui.image((tex.id(), scaled_size))
}
