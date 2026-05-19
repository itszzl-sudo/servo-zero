//! Mock 实现 - 用于测试和参考
//!
//! 使用独立的 layout-core，不依赖 SpiderMonkey

use std::collections::HashMap;
use layout_core::LayoutTree;
use crate::bridge::*;

/// Mock 实现 - 用于测试和参考
pub struct ServoBridge {
    width: u32,
    height: u32,
    layout_tree: LayoutTree,
    click_handlers: HashMap<String, EventHandler>,
    form_handlers: HashMap<String, FormHandler>,
    window_open_handler: Option<WindowOpenHandler>,
}

impl ServoBridge {
    /// 创建新的桥接器
    pub fn new_bridge(width: u32, height: u32) -> Self {
        let mut layout_tree = LayoutTree::new_empty();
        layout_tree.set_viewport(width as f32, height as f32);
        
        Self {
            width,
            height,
            layout_tree,
            click_handlers: HashMap::new(),
            form_handlers: HashMap::new(),
            window_open_handler: None,
        }
    }
    
    /// 设置 HTML（仅在 html feature 下可用）
    #[cfg(feature = "html")]
    pub fn set_html_internal(&mut self, html: &str) {
        let mut parser = html_core::parser::HtmlParser::new();
        let document = parser.parse(html);
        self.layout_tree = LayoutTree::new(document.clone());
        self.layout_tree.set_viewport(self.width as f32, self.height as f32);
        self.layout_tree.layout();
    }
    
    /// 获取 HTML 文档（仅在 html feature 下可用）
    #[cfg(feature = "html")]
    pub fn html_document(&self) -> &html_core::dom::HtmlDocument {
        self.layout_tree.document()
    }
}

#[cfg(feature = "html")]
mod html_support {
    use super::*;
    use html_core::dom::HtmlDocument;
    
    impl ServoBridge {
        /// 使用 HTML 文档创建桥接器
        pub fn with_document(document: HtmlDocument, width: u32, height: u32) -> Self {
            let mut layout_tree = LayoutTree::new(document);
            layout_tree.set_viewport(width as f32, height as f32);
            
            Self {
                width,
                height,
                layout_tree,
                click_handlers: HashMap::new(),
                form_handlers: HashMap::new(),
                window_open_handler: None,
            }
        }
    }
    
    impl WebNativeBridge for ServoBridge {
        fn new(width: u32, height: u32) -> Self {
            let document = HtmlDocument::new();
            let mut layout_tree = LayoutTree::new(document);
            layout_tree.set_viewport(width as f32, height as f32);
            
            Self {
                width,
                height,
                layout_tree,
                click_handlers: HashMap::new(),
                form_handlers: HashMap::new(),
                window_open_handler: None,
            }
        }
        
        fn set_html(&mut self, html: &str) {
            let mut parser = html_core::parser::HtmlParser::new();
            let document = parser.parse(html).clone();
            self.layout_tree = LayoutTree::new(document);
            self.layout_tree.set_viewport(self.width as f32, self.height as f32);
            self.layout_tree.layout();
        }
        
        fn query(&self, selector: &str) -> Option<usize> {
            self.layout_tree.document().query_selector(selector)
        }
        
        fn query_all(&self, selector: &str) -> Vec<usize> {
            self.layout_tree.document().query_selector_all(selector)
        }
        
        fn tag_name(&self, node_id: usize) -> Option<String> {
            self.layout_tree.document().get_node(node_id)
                .map(|n| n.tag_name_str.clone())
        }
        
        fn get_attr(&self, node_id: usize, name: &str) -> Option<String> {
            self.layout_tree.document().get_node(node_id)
                .and_then(|n| n.attributes.get(name).cloned())
        }
        
        fn set_attr(&mut self, node_id: usize, name: &str, value: &str) {
            if let Some(node) = self.layout_tree.document_mut().get_node_mut(node_id) {
                node.attributes.insert(name.to_string(), value.to_string());
            }
        }
        
        fn text(&self, node_id: usize) -> Option<String> {
            self.layout_tree.document().get_node(node_id)
                .map(|n| self.layout_tree.document().collect_text(n.id))
        }
        
        fn parent_node(&self, node_id: usize) -> Option<usize> {
            self.layout_tree.document().parent(node_id)
        }
        
        fn query_text(&self, selector: &str) -> Option<String> {
            let id = self.query(selector)?;
            self.text(id)
        }
        
        fn get_rect(&self, selector: &str) -> Option<LayoutRect> {
            self.layout_tree.get_rect(selector)
                .map(|r| LayoutRect {
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
        
        fn hit_test(&self, x: f32, y: f32) -> Option<LayoutNode> {
            self.layout_tree.hit_test(x, y)
                .map(|(id, tag, rect)| LayoutNode {
                    dom_node: id,
                    tag_name: tag,
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                    background: None,
                })
        }
        
        fn set_css(&mut self, css_text: &str) {
            self.layout_tree.add_css(css_text);
            self.layout_tree.layout();
        }
        
        fn set_style(&mut self, selector: &str, property: &str, value: &str) {
            self.layout_tree.set_element_style(selector, property, value);
            self.layout_tree.layout();
        }
        
        fn clear_css(&mut self) {
            let doc = self.layout_tree.document().clone();
            self.layout_tree = LayoutTree::new(doc);
            self.layout_tree.set_viewport(self.width as f32, self.height as f32);
            self.layout_tree.layout();
        }
        
        fn eval_js(&mut self, code: &str) -> String {
            log::warn!("eval_js called but JS is not supported in this build: {}", code);
            "undefined".to_string()
        }
        
        fn render(&mut self) -> Vec<u8> {
            let mut pixmap = tiny_skia::Pixmap::new(self.width, self.height)
                .unwrap_or_else(|| tiny_skia::Pixmap::new(800, 600).unwrap());
            pixmap.fill(tiny_skia::Color::WHITE);
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
                let selector = if let Some(id) = self.layout_tree.document().get_node(layout_node.dom_node)
                    .and_then(|n| n.attributes.get("id"))
                {
                    format!("#{}", id)
                } else {
                    layout_node.tag_name.clone()
                };
                
                if let Some(handler) = self.click_handlers.get_mut(&selector) {
                    handler(x, y);
                    return true;
                }
            }
            false
        }
        
        fn handle_form_submit(&mut self, form_selector: &str) {
            if let Some(handler) = self.form_handlers.get_mut(form_selector) {
                handler(HashMap::new());
            }
        }
        
        fn handle_window_open(&mut self, url: &str) -> bool {
            if let Some(ref mut handler) = self.window_open_handler {
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
}

#[cfg(not(feature = "html"))]
mod no_html_support {
    use super::*;
    
    impl WebNativeBridge for ServoBridge {
        fn new(width: u32, height: u32) -> Self {
            let mut layout_tree = LayoutTree::new_empty();
            layout_tree.set_viewport(width as f32, height as f32);
            
            Self {
                width,
                height,
                layout_tree,
                click_handlers: HashMap::new(),
                form_handlers: HashMap::new(),
                window_open_handler: None,
            }
        }
        
        fn get_rect(&self, selector: &str) -> Option<LayoutRect> {
            self.layout_tree.get_rect(selector)
                .map(|r| LayoutRect {
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
        
        fn hit_test(&self, x: f32, y: f32) -> Option<LayoutNode> {
            self.layout_tree.hit_test(x, y)
                .map(|(id, tag, rect)| LayoutNode {
                    dom_node: id,
                    tag_name: tag,
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                    background: None,
                })
        }
        
        fn set_css(&mut self, css_text: &str) {
            self.layout_tree.add_css(css_text);
            self.layout_tree.layout();
        }
        
        fn set_style(&mut self, selector: &str, property: &str, value: &str) {
            self.layout_tree.set_element_style(selector, property, value);
            self.layout_tree.layout();
        }
        
        fn clear_css(&mut self) {
            self.layout_tree = LayoutTree::new_empty();
            self.layout_tree.set_viewport(self.width as f32, self.height as f32);
            self.layout_tree.layout();
        }
        
        fn eval_js(&mut self, code: &str) -> String {
            log::warn!("eval_js called but JS is not supported in this build: {}", code);
            "undefined".to_string()
        }
        
        fn render(&mut self) -> Vec<u8> {
            let mut pixmap = tiny_skia::Pixmap::new(self.width, self.height)
                .unwrap_or_else(|| tiny_skia::Pixmap::new(800, 600).unwrap());
            pixmap.fill(tiny_skia::Color::WHITE);
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
        
        fn handle_form_submit(&mut self, form_selector: &str) {
            if let Some(handler) = self.form_handlers.get_mut(form_selector) {
                handler(HashMap::new());
            }
        }
        
        fn handle_window_open(&mut self, url: &str) -> bool {
            if let Some(ref mut handler) = self.window_open_handler {
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
}
