//! 真实实现 - 使用独立的核心组件
//!
//! 支持 CSS 布局和 tiny-skia 渲染，不依赖 SpiderMonkey

use std::collections::HashMap;
use std::sync::OnceLock;
use layout_core::LayoutTree;
use html_core::parser::{HtmlParser, extract_style_tags};
use crate::bridge::*;
use fontdue::{Font, FontSettings};

pub(crate) static DEFAULT_FONT: OnceLock<Option<Font>> = OnceLock::new();

pub(crate) fn get_default_font() -> Option<&'static Font> {
    DEFAULT_FONT.get_or_init(|| {
        let font_paths = [
            "C:/Windows/Fonts/arial.ttf",
            "C:/Windows/Fonts/segoeui.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
        ];
        
        for path in &font_paths {
            if let Ok(data) = std::fs::read(path) {
                if let Ok(font) = Font::from_bytes(data, FontSettings::default()) {
                    log::info!("加载字体成功: {}", path);
                    return Some(font);
                }
            }
        }
        
        log::warn!("未找到系统字体，文本渲染将禁用");
        None
    }).as_ref()
}

pub(crate) fn render_text(
    pixmap: &mut tiny_skia::Pixmap,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    color: (u8, u8, u8, u8),
) {
    let font = match get_default_font() {
        Some(f) => f,
        None => return,
    };
    
    let mut current_x = x;
    
    // 使用fontdue的正确API
    for c in text.chars() {
        let (metrics, bitmap) = font.rasterize(c, font_size);
        
        if metrics.width > 0 && metrics.height > 0 {
            let glyph_x = current_x + metrics.xmin as f32;
            let glyph_y = y - metrics.ymin as f32;
            
            for py in 0..metrics.height {
                for px in 0..metrics.width {
                    let alpha = bitmap[py * metrics.width + px] as f32 / 255.0;
                    if alpha > 0.01 {
                        let pixel_x = (glyph_x + px as f32) as u32;
                        let pixel_y = (glyph_y + py as f32) as u32;
                        
                        if pixel_x < pixmap.width() && pixel_y < pixmap.height() {
                            let idx = (pixel_y * pixmap.width() + pixel_x) as usize * 4;
                            let data = pixmap.data();
                            
                            // 读取背景色
                            let bg_r = data[idx] as f32 / 255.0;
                            let bg_g = data[idx + 1] as f32 / 255.0;
                            let bg_b = data[idx + 2] as f32 / 255.0;
                            let bg_a = data[idx + 3] as f32 / 255.0;
                            
                            let fg_r = color.0 as f32 / 255.0;
                            let fg_g = color.1 as f32 / 255.0;
                            let fg_b = color.2 as f32 / 255.0;
                            let fg_a = color.3 as f32 / 255.0;
                            
                            let out_a = fg_a * alpha + bg_a * (1.0 - fg_a * alpha);
                            if out_a > 0.001 {
                                let out_r = (fg_r * fg_a * alpha + bg_r * bg_a * (1.0 - fg_a * alpha)) / out_a;
                                let out_g = (fg_g * fg_a * alpha + bg_g * bg_a * (1.0 - fg_a * alpha)) / out_a;
                                let out_b = (fg_b * fg_a * alpha + bg_b * bg_a * (1.0 - fg_a * alpha)) / out_a;
                                
                                // 现在修改像素
                                let data_mut = pixmap.data_mut();
                                data_mut[idx] = (out_r * 255.0) as u8;
                                data_mut[idx + 1] = (out_g * 255.0) as u8;
                                data_mut[idx + 2] = (out_b * 255.0) as u8;
                                data_mut[idx + 3] = (out_a * 255.0) as u8;
                            }
                        }
                    }
                }
            }
        }
        
        current_x += metrics.advance_width;
    }
}

/// 真实桥接实现
pub struct RealServoBridge {
    width: u32,
    height: u32,
    layout_tree: LayoutTree,
    css_rules: Vec<String>,
    click_handlers: HashMap<String, EventHandler>,
    form_handlers: HashMap<String, FormHandler>,
    window_open_handler: Option<WindowOpenHandler>,
}

impl RealServoBridge {
    /// 创建新的桥接器
    pub fn new_bridge(width: u32, height: u32) -> Self {
        let mut layout_tree = LayoutTree::new_empty();
        layout_tree.set_viewport(width as f32, height as f32);

        Self {
            width,
            height,
            layout_tree,
            css_rules: Vec::new(),
            click_handlers: HashMap::new(),
            form_handlers: HashMap::new(),
            window_open_handler: None,
        }
    }

    /// 解析内联样式并应用到元素
    fn apply_inline_styles(&mut self, _html: &str) {
        // 简单的内联样式解析：查找 style="..." 属性
        // 这里我们依赖HTML解析器已经解析了style属性
        // 只需要确保布局树能正确读取它们
    
        log::info!("apply_inline_styles: 开始解析内联样式");
    
        // 遍历DOM树中的所有元素
        let mut node_ids: Vec<usize> = Vec::new();
    
        // 收集所有节点ID
        if let Some(root_id) = self.layout_tree.document().root_id() {
            node_ids.push(root_id);
            let mut idx = 0;
            while idx < node_ids.len() {
                let current_id = node_ids[idx];
                if let Some(node) = self.layout_tree.document().get_node(current_id) {
                    for &child_id in &node.children {
                        node_ids.push(child_id);
                    }
                }
                idx += 1;
            }
        }
    
        log::info!("apply_inline_styles: 遍历了 {} 个节点", node_ids.len());
    
        // 为每个元素应用内联样式
        // 先收集所有样式信息，避免借用冲突
        let mut styles_to_apply: Vec<(String, Vec<(String, String)>)> = Vec::new();
    
        for node_id in node_ids {
            if let Some(node) = self.layout_tree.document().get_node(node_id) {
                // 跳过非元素节点
                if node.node_type != html_core::dom::DomNodeType::Element {
                    continue;
                }
    
                if let Some(style_attr) = node.get_attr("style") {
                    log::debug!("apply_inline_styles: 节点 {} ({}) 有内联样式: {}",
                        node_id, node.tag_name_str, style_attr);
    
                    // 构建选择器：使用 __nid_<id> 作为唯一 key，与 cascade.rs 对齐
                    let selector = format!("__nid_{}", node_id);
    
                    log::debug!("apply_inline_styles: 选择器: {}", selector);
    
                    // 解析样式属性
                    let declarations: Vec<(String, String)> = style_attr
                        .split(';')
                        .filter_map(|decl| {
                            let parts: Vec<&str> = decl.splitn(2, ':').collect();
                            if parts.len() == 2 {
                                let prop = parts[0].trim().to_string();
                                let value = parts[1].trim().to_string();
                                if !prop.is_empty() && !value.is_empty() {
                                    Some((prop, value))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect();
    
                    log::debug!("apply_inline_styles: 解析到 {} 个声明", declarations.len());
    
                    if !declarations.is_empty() {
                        for (prop, value) in &declarations {
                            log::debug!("apply_inline_styles:   - {}: {}", prop, value);
                        }
                        styles_to_apply.push((selector, declarations));
                    }
                }
            }
        }
    
        // 应用收集到的样式
        log::info!("apply_inline_styles: 将应用 {} 个样式块", styles_to_apply.len());
        for (selector, declarations) in styles_to_apply {
            for (property, value) in declarations {
                self.layout_tree.set_element_style(&selector, &property, &value);
            }
        }
    }
    
    /// 渲染带换行的文本
    fn render_text_wrapped(
        &self,
        pixmap: &mut tiny_skia::Pixmap,
        text: &str,
        x: f32,
        y: f32,
        container_width: f32,
        font_size: f32,
        color: (u8, u8, u8, u8),
        text_align: &Option<String>,
    ) {
        if container_width <= 0.0 {
            return;
        }
    
        let font = match get_default_font() {
            Some(f) => f,
            None => return,
        };
    
        // 按词拆分
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut lines: Vec<String> = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0.0f32;
    
        for word in words {
            let word_width = self.measure_text_width(font, word, font_size);
            let space_width = self.measure_text_width(font, " ", font_size);
    
            if !current_line.is_empty() && current_width + space_width + word_width > container_width {
                // 换行
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                }
                current_line = word.to_string();
                current_width = word_width;
            } else {
                if !current_line.is_empty() {
                    current_line.push(' ');
                    current_width += space_width;
                }
                current_line.push_str(word);
                current_width += word_width;
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }
    
        // 渲染每一行
        for (i, line) in lines.iter().enumerate() {
            let line_width = self.measure_text_width(font, line, font_size);
            let line_x = match text_align.as_deref() {
                Some("center") => x + (container_width - line_width) / 2.0,
                Some("right") => x + container_width - line_width,
                _ => x,
            };
            let line_y = y + i as f32 * (font_size * 1.3);
    
            render_text(pixmap, line, line_x.max(x), line_y, font_size, color);
        }
    }
    
    /// 测量文本宽度
    fn measure_text_width(&self, font: &Font, text: &str, font_size: f32) -> f32 {
        let mut width = 0.0f32;
        for c in text.chars() {
            let (metrics, _) = font.rasterize(c, font_size);
            width += metrics.advance_width;
        }
        width
    }
}

impl WebNativeBridge for RealServoBridge {
    fn new(width: u32, height: u32) -> Self {
        let mut layout_tree = LayoutTree::new_empty();
        layout_tree.set_viewport(width as f32, height as f32);
        
        Self {
            width,
            height,
            layout_tree,
            css_rules: Vec::new(),
            click_handlers: HashMap::new(),
            form_handlers: HashMap::new(),
            window_open_handler: None,
        }
    }

    fn get_rect(&self, selector: &str) -> Option<LayoutRect> {
        self.layout_tree.get_rect(selector).map(|r| LayoutRect {
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
        })
    }

    fn all_rects(&self) -> Vec<LayoutNode> {
        self.layout_tree.all_rects()
            .into_iter()
            .map(|(id, tag, rect, bg)| LayoutNode {
                dom_node: id,
                tag_name: tag,
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
                background: bg.map(|(r, g, b, a)| Color { r, g, b, a }),
            })
            .collect()
    }
    
    fn set_html(&mut self, html: &str) {
        // 1. 解析 HTML（使用 html5ever）
        let mut parser = HtmlParser::new();
        let document = parser.parse(html).clone();
        
        log::info!("HTML解析完成，节点数量: {}", document.nodes_len());
        
        // 2. 提取 <style> 标签中的 CSS（同样用 html5ever 解析）
        let css_styles = extract_style_tags(html);
        
        // 3. 重建布局树
        self.layout_tree = LayoutTree::new(document);
        self.layout_tree.set_viewport(self.width as f32, self.height as f32);
        
        // 4. 应用提取的 CSS
        for css in &css_styles {
            self.layout_tree.add_css(css);
        }
        
        // 5. 应用之前通过 set_css 添加的规则
        for css in &self.css_rules {
            self.layout_tree.add_css(css);
        }
        
        // 6. 解析内联样式并应用到元素
        self.apply_inline_styles(html);
        
        // 7. 触发布局计算
        self.layout_tree.layout();
        
        log::info!("布局计算完成，布局节点数量: {}", self.layout_tree.all_rects().len());
    }

    fn hit_test(&self, x: f32, y: f32) -> Option<LayoutNode> {
        self.layout_tree.hit_test(x, y).map(|(id, tag, rect)| LayoutNode {
            dom_node: id,
            tag_name: tag,
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
            background: self.layout_tree.get_box(id)
                .and_then(|b| b.node.background)
                .map(|(r, g, b, a)| Color { r, g, b, a }),
        })
    }

    fn set_css(&mut self, css_text: &str) {
        self.css_rules.push(css_text.to_string());
        self.layout_tree.add_css(css_text);
        self.layout_tree.layout();
    }

    fn set_style(&mut self, selector: &str, property: &str, value: &str) {
        self.layout_tree.set_element_style(selector, property, value);
        self.layout_tree.layout();
    }

    fn clear_css(&mut self) {
        self.css_rules.clear();
        self.layout_tree = LayoutTree::new_empty();
        self.layout_tree.set_viewport(self.width as f32, self.height as f32);
        self.layout_tree.layout();
    }

    fn eval_js(&mut self, _code: &str) -> String {
        log::warn!("eval_js called but JS is not supported");
        String::new()
    }

    fn render(&mut self) -> Vec<u8> {
        let mut pixmap = tiny_skia::Pixmap::new(self.width, self.height)
            .unwrap_or_else(|| tiny_skia::Pixmap::new(800, 600).unwrap());
        
        // 填充白色背景
        pixmap.fill(tiny_skia::Color::WHITE);
        
        // 收集所有需要渲染的节点
        let mut nodes: Vec<_> = self.layout_tree.all_rects().into_iter().collect();
        
        log::info!("渲染节点数量: {}", nodes.len());
        
        // 按深度排序：父元素先渲染，子元素后渲染（从外到内）
        // 这样子元素会覆盖父元素
        nodes.sort_by(|a, b| {
            let (_, _, rect_a, _) = a;
            let (_, _, rect_b, _) = b;
            
            // 按 y 坐标排序，y 小的先渲染
            // 同一 y 坐标，按 x 排序
            // 同一位置，小尺寸（子元素）后渲染
            if rect_a.y != rect_b.y {
                rect_a.y.partial_cmp(&rect_b.y).unwrap()
            } else if rect_a.x != rect_b.x {
                rect_a.x.partial_cmp(&rect_b.x).unwrap()
            } else {
                // 尺寸大的先渲染（父元素通常比子元素大）
                let area_a = rect_a.width * rect_a.height;
                let area_b = rect_b.width * rect_b.height;
                area_b.partial_cmp(&area_a).unwrap()
            }
        });
        
        // 绘制所有布局节点
        let mut paint = tiny_skia::Paint::default();
        for (id, tag, rect, bg) in &nodes {
            // 没有显式设置背景色的元素不绘制色块（避免覆盖其他内容）
            let color = match bg {
                Some(c) => *c,
                None => {
                    // 即使没有背景色，也可能有 border，让入下面的 border 处理
                    if let Some(box_info) = self.layout_tree.get_box(*id) {
                        if let (Some(bw), Some(bc)) = (box_info.node.border_width, box_info.node.border_color) {
                            if bw > 0.0 {
                                let stroke = tiny_skia::Stroke { width: bw, ..Default::default() };
                                let mut border_paint = tiny_skia::Paint::default();
                                border_paint.set_color_rgba8(bc.0, bc.1, bc.2, bc.3);
                                if let Some(sk_rect) = tiny_skia::Rect::from_xywh(rect.x, rect.y, rect.width, rect.height) {
                                    let path = tiny_skia::PathBuilder::from_rect(sk_rect);
                                    pixmap.stroke_path(&path, &border_paint, &stroke, tiny_skia::Transform::identity(), None);
                                }
                            }
                        }
                    }
                    continue;
                }
            };
            
            // 确保尺寸有效
            if rect.width <= 0.0 || rect.height <= 0.0 {
                log::warn!("节点 {} ({}) 尺寸无效: {}x{}", id, tag, rect.width, rect.height);
                continue;
            }
            
            paint.set_color_rgba8(color.0, color.1, color.2, color.3);
            
            if let Some(tiny_skia_rect) = tiny_skia::Rect::from_xywh(rect.x, rect.y, rect.width, rect.height) {
                let path = tiny_skia::PathBuilder::from_rect(tiny_skia_rect);
                
                // 应用 CSS transform
                let transform = if let Some(box_info) = self.layout_tree.get_box(*id) {
                    if let Some(matrix) = box_info.node.transform_matrix {
                        // 从矩阵创建 tiny_skia::Transform
                        // tiny_skia 使用 from_row(sx, ky, kx, sy, tx, ty)
                        tiny_skia::Transform::from_row(matrix[0], matrix[1], matrix[2], matrix[3], matrix[4], matrix[5])
                    } else {
                        tiny_skia::Transform::identity()
                    }
                } else {
                    tiny_skia::Transform::identity()
                };
                                
                pixmap.fill_path(
                    &path,
                    &paint,
                    tiny_skia::FillRule::Winding,
                    transform,
                    None,
                );
                                
                // 背景色绘制后再画 border
                if let Some(box_info) = self.layout_tree.get_box(*id) {
                    if let (Some(bw), Some(bc)) = (box_info.node.border_width, box_info.node.border_color) {
                        if bw > 0.0 {
                            let stroke = tiny_skia::Stroke { width: bw, ..Default::default() };
                            let mut border_paint = tiny_skia::Paint::default();
                            border_paint.set_color_rgba8(bc.0, bc.1, bc.2, bc.3);
                            pixmap.stroke_path(&path, &border_paint, &stroke, transform, None);
                        }
                    }
                }
                                
                log::debug!("绘制节点 {} ({}): ({}, {}, {}, {}), 颜色: {:?}", 
                    id, tag, rect.x, rect.y, rect.width, rect.height, color);
            }
        }
        
        // 渲染文本节点
        if let Some(root_id) = self.layout_tree.document().root_id() {
            let mut text_nodes = Vec::new();
            let mut stack = vec![root_id];
        
            while let Some(node_id) = stack.pop() {
                if let Some(node) = self.layout_tree.document().get_node(node_id) {
                    stack.extend(node.children.iter().rev());
        
                    if node.node_type == html_core::dom::DomNodeType::Text {
                        if let Some(ref text) = node.text_content {
                            let trimmed = text.trim();
                            if !trimmed.is_empty() {
                                text_nodes.push((node_id, trimmed.to_string()));
                            }
                        }
                    }
                }
            }
        
            log::info!("找到 {} 个文本节点", text_nodes.len());
        
            for (node_id, text) in text_nodes {
                if let Some(parent_id) = self.layout_tree.document().get_node(node_id)
                    .and_then(|n| n.parent)
                {
                    if let Some(parent_box) = self.layout_tree.get_box(parent_id) {
                        let rect = parent_box.content_box;
        
                        // 从 CSS font-size 属性获取字体大小，默认 14px
                        let font_size = parent_box.node.font_size.unwrap_or(14.0);
                        
                        // 计算文本高度（根据行数）
                        let line_height = font_size * 1.3;
                        let text_height = if rect.width > 0.0 {
                            let font = match get_default_font() {
                                Some(f) => f,
                                None => continue,
                            };
                            let text_width = self.measure_text_width(font, &text, font_size);
                            let lines = ((text_width / rect.width).ceil() as usize).max(1);
                            lines as f32 * line_height
                        } else {
                            line_height
                        };
        
                        // 计算 padding 偏移
                        let pad_left = parent_box.padding_box.x - parent_box.border_box.x
                            + (parent_box.content_box.x - parent_box.padding_box.x);
                        let pad_top = parent_box.padding_box.y - parent_box.border_box.y
                            + (parent_box.content_box.y - parent_box.padding_box.y);
        
                        let container_x = rect.x + pad_left;
                        let container_w = rect.width - pad_left * 2.0;
                        
                        // 修正容器宽度（如果为0则使用文本宽度）
                        let container_w = if container_w > 0.0 {
                            container_w
                        } else {
                            let font = match get_default_font() {
                                Some(f) => f,
                                None => continue,
                            };
                            self.measure_text_width(font, &text, font_size) + 20.0
                        };
                        
                        // 文本 Y 坐标：考虑修正后的高度
                        let text_y = if rect.height < text_height {
                            rect.y + pad_top + font_size + 2.0
                        } else {
                            rect.y + pad_top + font_size + 2.0
                        };
        
                        // 文本换行渲染
                        self.render_text_wrapped(
                            &mut pixmap,
                            &text,
                            container_x,
                            text_y,
                            container_w.max(0.0),
                            font_size,
                            parent_box.node.color.unwrap_or((0, 0, 0, 255)),
                            &parent_box.node.text_align,
                        );
                    }
                }
            }
        }
        
        log::info!("渲染完成，PNG大小: {} bytes", 
            pixmap.encode_png().unwrap_or_default().len());
        
        pixmap.encode_png().unwrap_or_default()
    }

    fn on_click(&mut self, selector: &str, handler: EventHandler) {
        self.click_handlers.insert(selector.to_string(), handler);
    }

    fn remove_on_click(&mut self, selector: &str) -> bool {
        self.click_handlers.remove(selector).is_some()
    }

    fn on_form_submit(&mut self, selector: &str, handler: FormHandler) {
        self.form_handlers.insert(selector.to_string(), handler);
    }

    fn remove_on_form_submit(&mut self, selector: &str) -> bool {
        self.form_handlers.remove(selector).is_some()
    }

    fn on_window_open(&mut self, handler: WindowOpenHandler) {
        self.window_open_handler = Some(handler);
    }

    fn remove_on_window_open(&mut self) -> bool {
        self.window_open_handler.take().is_some()
    }

    fn handle_click(&mut self, x: f32, y: f32) -> bool {
        if let Some(layout_node) = self.hit_test(x, y) {
            // 1. 尝试 #id 选择器（最高优先级）
            let id_selector = self.layout_tree.document()
                .get_node(layout_node.dom_node)
                .and_then(|n| n.attributes.get("id"))
                .map(|id| format!("#{}", id));

            if let Some(ref sel) = id_selector {
                if let Some(handler) = self.click_handlers.get_mut(sel) {
                    handler(x, y);
                    return true;
                }
            }

            // 2. 尝试 tag_name 选择器
            if let Some(handler) = self.click_handlers.get_mut(&layout_node.tag_name) {
                handler(x, y);
                return true;
            }

            // 3. 尝试 .class 选择器
            if let Some(node) = self.layout_tree.document().get_node(layout_node.dom_node) {
                if let Some(class_attr) = node.get_attr("class") {
                    for class in class_attr.split_whitespace() {
                        let class_selector = format!(".{}", class);
                        if let Some(handler) = self.click_handlers.get_mut(&class_selector) {
                            handler(x, y);
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn handle_form_submit(&mut self, _form_selector: &str) {}

    fn handle_window_open(&mut self, url: &str) -> bool {
        if let Some(handler) = &mut self.window_open_handler {
            handler(url)
        } else {
            false
        }
    }

    fn set_viewport(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.layout_tree.set_viewport(width as f32, height as f32);
        self.layout_tree.layout();
    }

    fn viewport(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    // ── 网络请求（需要 network feature） ──

    #[cfg(feature = "network")]
    fn navigate(&mut self, url: &str) -> Result<(), String> {
        let response = reqwest::blocking::get(url)
            .map_err(|e| format!("Network error: {}", e))?;
        
        let body = response.bytes()
            .map_err(|e| format!("Body error: {}", e))?;
        
        let html = String::from_utf8_lossy(&body).to_string();
        self.set_html(&html);
        
        Ok(())
    }

    #[cfg(not(feature = "network"))]
    fn navigate(&mut self, _url: &str) -> Result<(), String> {
        Err("Network feature not enabled".to_string())
    }

    #[cfg(feature = "network")]
    fn current_url(&self) -> String {
        String::new()
    }

    #[cfg(not(feature = "network"))]
    fn current_url(&self) -> String {
        String::new()
    }

    #[cfg(feature = "network")]
    fn http_get(&mut self, url: &str) -> Result<crate::network::HttpResponse, String> {
        let response = reqwest::blocking::get(url)
            .map_err(|e| format!("HTTP GET error: {}", e))?;
        
        let status = response.status().as_u16();
        let headers: HashMap<String, String> = response.headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let body = response.bytes()
            .map_err(|e| format!("Body error: {}", e))?
            .to_vec();
        
        Ok(crate::network::HttpResponse {
            status,
            headers,
            body,
            url: url.to_string(),
        })
    }

    #[cfg(not(feature = "network"))]
    fn http_get(&mut self, _url: &str) -> Result<crate::network::HttpResponse, String> {
        Err("Network feature not enabled".to_string())
    }

    #[cfg(feature = "network")]
    fn http_post(&mut self, url: &str, body: &[u8], content_type: &str) -> Result<crate::network::HttpResponse, String> {
        let client = reqwest::blocking::Client::new();
        
        let response = client
            .post(url)
            .header("Content-Type", content_type)
            .body(body.to_vec())
            .send()
            .map_err(|e| format!("HTTP POST error: {}", e))?;
        
        let status = response.status().as_u16();
        let headers: HashMap<String, String> = response.headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let resp_body = response.bytes()
            .map_err(|e| format!("Body error: {}", e))?
            .to_vec();
        
        Ok(crate::network::HttpResponse {
            status,
            headers,
            body: resp_body,
            url: url.to_string(),
        })
    }

    #[cfg(not(feature = "network"))]
    fn http_post(&mut self, _url: &str, _body: &[u8], _content_type: &str) -> Result<crate::network::HttpResponse, String> {
        Err("Network feature not enabled".to_string())
    }

    #[cfg(feature = "network")]
    fn download_file(&mut self, url: &str, path: &str) -> Result<u64, String> {
        let response = reqwest::blocking::get(url)
            .map_err(|e| format!("Download error: {}", e))?;
        
        let body = response.bytes()
            .map_err(|e| format!("Body error: {}", e))?;
        
        std::fs::write(path, &body)
            .map_err(|e| format!("Write error: {}", e))?;
        
        Ok(body.len() as u64)
    }

    #[cfg(not(feature = "network"))]
    fn download_file(&mut self, _url: &str, _path: &str) -> Result<u64, String> {
        Err("Network feature not enabled".to_string())
    }

    // ── 文件操作 ──

    fn write_file(&mut self, path: &str, data: &[u8]) -> Result<(), String> {
        std::fs::write(path, data)
            .map_err(|e| format!("Write error: {}", e))
    }

    fn read_file(&mut self, path: &str) -> Result<Vec<u8>, String> {
        std::fs::read(path)
            .map_err(|e| format!("Read error: {}", e))
    }
}
