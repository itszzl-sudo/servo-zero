//! 真实实现 - 使用独立的核心组件
//!
//! 支持 CSS 布局和 tiny-skia 渲染，不依赖 SpiderMonkey

use std::collections::HashMap;
use layout_core::LayoutTree;
use crate::bridge::*;

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

    fn get_rect(&self, _selector: &str) -> Option<LayoutRect> {
        Some(LayoutRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
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

    fn hit_test(&self, _x: f32, _y: f32) -> Option<LayoutNode> {
        None
    }

    fn set_css(&mut self, css_text: &str) {
        self.css_rules.push(css_text.to_string());
        self.layout_tree.add_css(css_text);
        self.layout_tree.layout();
    }

    fn set_style(&mut self, _selector: &str, _property: &str, _value: &str) {}

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
        
        pixmap.fill(tiny_skia::Color::WHITE);
        
        // 绘制所有布局节点
        let mut paint = tiny_skia::Paint::default();
        for (_id, _tag, rect, bg) in self.layout_tree.all_rects() {
            if let Some((r, g, b, a)) = bg {
                paint.set_color_rgba8(r, g, b, a);
                
                let path = tiny_skia::PathBuilder::from_rect(
                    tiny_skia::Rect::from_xywh(rect.x, rect.y, rect.width, rect.height).unwrap()
                );
                
                pixmap.fill_path(
                    &path,
                    &paint,
                    tiny_skia::FillRule::Winding,
                    tiny_skia::Transform::identity(),
                    None,
                );
            }
        }
        
        pixmap.encode_png().unwrap_or_default()
    }

    fn on_click(&mut self, selector: &str, handler: EventHandler) {
        self.click_handlers.insert(selector.to_string(), handler);
    }

    fn on_form_submit(&mut self, selector: &str, handler: FormHandler) {
        self.form_handlers.insert(selector.to_string(), handler);
    }

    fn on_window_open(&mut self, handler: WindowOpenHandler) {
        self.window_open_handler = Some(handler);
    }

    fn handle_click(&mut self, x: f32, y: f32) -> bool {
        if let Some(layout_node) = self.hit_test(x, y) {
            if let Some(handler) = self.click_handlers.get_mut(&layout_node.tag_name) {
                handler(x, y);
                return true;
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

    fn write_file(&mut self, path: &str, data: &[u8]) -> Result<(), String> {
        std::fs::write(path, data)
            .map_err(|e| format!("Write error: {}", e))
    }

    fn read_file(&mut self, path: &str) -> Result<Vec<u8>, String> {
        std::fs::read(path)
            .map_err(|e| format!("Read error: {}", e))
    }
}
