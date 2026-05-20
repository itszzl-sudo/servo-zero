//! Mock 实现 - 用于测试和参考
//!
//! 使用独立的 layout-core，不依赖 SpiderMonkey

use std::collections::HashMap;
use layout_core::LayoutTree;
use crate::bridge::*;
use crate::real_impl::render_text;

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

    /// 实现真正的页面渲染（返回 PNG 字节）
    fn do_render(&mut self) -> Vec<u8> {
        let mut pixmap = tiny_skia::Pixmap::new(self.width, self.height)
            .unwrap_or_else(|| tiny_skia::Pixmap::new(800, 600).unwrap());

        // 白色背景
        pixmap.fill(tiny_skia::Color::WHITE);

        // 收集所有需要渲染的节点
        let mut nodes: Vec<_> = self.layout_tree.all_rects().into_iter().collect();

        // 尺寸大的先渲染（父元素）
        nodes.sort_by(|a, b| {
            let area_a = a.2.width * a.2.height;
            let area_b = b.2.width * b.2.height;
            area_b.partial_cmp(&area_a).unwrap()
        });

        let mut paint = tiny_skia::Paint::default();
        for (id, tag, rect, bg) in &nodes {
            let color = match bg {
                Some(c) => *c,
                None => {
                    // 即使没有背景色，也可能有 border
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

            if rect.width <= 0.0 || rect.height <= 0.0 { continue; }

            paint.set_color_rgba8(color.0, color.1, color.2, color.3);
            if let Some(sk_rect) = tiny_skia::Rect::from_xywh(rect.x, rect.y, rect.width, rect.height) {
                let path = tiny_skia::PathBuilder::from_rect(sk_rect);
                pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding,
                    tiny_skia::Transform::identity(), None);

                // 画 border
                if let Some(box_info) = self.layout_tree.get_box(*id) {
                    if let (Some(bw), Some(bc)) = (box_info.node.border_width, box_info.node.border_color) {
                        if bw > 0.0 {
                            let stroke = tiny_skia::Stroke { width: bw, ..Default::default() };
                            let mut border_paint = tiny_skia::Paint::default();
                            border_paint.set_color_rgba8(bc.0, bc.1, bc.2, bc.3);
                            pixmap.stroke_path(&path, &border_paint, &stroke, tiny_skia::Transform::identity(), None);
                        }
                    }
                }
            }
            let _ = tag; // 消除未用变量警告
        }

        // 渲染文本节点
        if let Some(root_id) = self.layout_tree.document().root_id() {
            let mut stack = vec![root_id];
            let mut text_nodes = Vec::new();

            while let Some(node_id) = stack.pop() {
                if let Some(node) = self.layout_tree.document().get_node(node_id) {
                    stack.extend(node.children.iter().rev());
                    if node.node_type == html_core::dom::DomNodeType::Text {
                        if let Some(ref t) = node.text_content {
                            let trimmed = t.trim();
                            if !trimmed.is_empty() {
                                text_nodes.push((node_id, trimmed.to_string()));
                            }
                        }
                    }
                }
            }

            for (node_id, text) in text_nodes {
                if let Some(parent_id) = self.layout_tree.document().get_node(node_id).and_then(|n| n.parent) {
                    if let Some(parent_box) = self.layout_tree.get_box(parent_id) {
                        let rect = parent_box.content_box;
                        let font_size = parent_box.node.font_size.unwrap_or(14.0);
                        let text_width_est = text.chars().count() as f32 * font_size * 0.55;
                        let text_x = match parent_box.node.text_align.as_deref() {
                            Some("center") => (rect.x + (rect.width - text_width_est) / 2.0).max(rect.x),
                            Some("right")  => (rect.x + rect.width - text_width_est - 4.0).max(rect.x + 4.0),
                            _ => rect.x + 4.0,
                        };
                        let text_y = rect.y + font_size + 2.0;
                        let text_color = parent_box.node.color.unwrap_or((0, 0, 0, 255));
                        render_text(&mut pixmap, &text, text_x, text_y, font_size, text_color);
                    }
                }
            }
        }

        pixmap.encode_png().unwrap_or_default()
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
            self.do_render()
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

        // 网络方法（mock 实现）
        fn navigate(&mut self, _url: &str) -> Result<(), String> {
            Err("Network not supported in ServoBridge mock".to_string())
        }

        fn current_url(&self) -> String { String::new() }

        fn http_get(&mut self, _url: &str) -> Result<crate::network::HttpResponse, String> {
            Err("Network not supported in ServoBridge mock".to_string())
        }

        fn http_post(&mut self, _url: &str, _body: &[u8], _content_type: &str) -> Result<crate::network::HttpResponse, String> {
            Err("Network not supported in ServoBridge mock".to_string())
        }

        fn download_file(&mut self, _url: &str, _path: &str) -> Result<u64, String> {
            Err("Network not supported in ServoBridge mock".to_string())
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
            self.do_render()
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

        // 网络方法（mock 实现，不支持实际请求）
        fn navigate(&mut self, _url: &str) -> Result<(), String> {
            Err("Network not supported in ServoBridge mock".to_string())
        }

        fn current_url(&self) -> String { String::new() }

        fn http_get(&mut self, _url: &str) -> Result<crate::network::HttpResponse, String> {
            Err("Network not supported in ServoBridge mock".to_string())
        }

        fn http_post(&mut self, _url: &str, _body: &[u8], _content_type: &str) -> Result<crate::network::HttpResponse, String> {
            Err("Network not supported in ServoBridge mock".to_string())
        }

        fn download_file(&mut self, _url: &str, _path: &str) -> Result<u64, String> {
            Err("Network not supported in ServoBridge mock".to_string())
        }
    }
}
