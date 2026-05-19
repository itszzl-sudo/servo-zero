//! 默认的 Mock 实现，供测试和参考使用

use std::collections::HashMap;

use crate::bridge::*;
use crate::network::HttpResponse;

/// Mock 实现 - 用于测试和参考
pub struct ServoBridge {
    width: u32,
    height: u32,
    css_rules: Vec<String>,
    styles: HashMap<String, Vec<(String, String)>>,
    click_handlers: HashMap<String, EventHandler>,
    form_handlers: HashMap<String, FormHandler>,
    window_open_handler: Option<WindowOpenHandler>,
    click_log: Vec<String>,
    form_log: Vec<String>,
    js_log: Vec<String>,
    js_return: String,
    nodes: Vec<MockNode>,
}

#[derive(Clone)]
struct MockNode {
    id: usize,
    tag: String,
    text: String,
    attrs: HashMap<String, String>,
    parent: Option<usize>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    bg: Option<Color>,
}

impl ServoBridge {
    fn parse_html(&mut self, html: &str) {
        self.nodes.clear();
        let mut id_counter = 1usize;
        let mut rest = html;
        while let Some(tag_start) = rest.find('<') {
            let tag_end = rest[tag_start..].find('>').map(|p| tag_start + p + 1);
            match tag_end {
                Some(end) => {
                    let tag_content = &rest[tag_start + 1..end - 1];
                    if tag_content.starts_with("!--") || tag_content.starts_with('/') {
                        rest = &rest[end..];
                        continue;
                    }
                    let parts: Vec<&str> = tag_content.split_whitespace().collect();
                    let tag_name = parts.first().map(|s| s.to_string()).unwrap_or_default();
                    let mut attrs = HashMap::new();
                    for part in parts.iter().skip(1) {
                        if let Some((k, v)) = part.split_once('=') {
                            let val = v.trim_matches('"');
                            attrs.insert(k.to_string(), val.to_string());
                        }
                    }
                    let node_id = id_counter;
                    id_counter += 1;
                    self.nodes.push(MockNode {
                        id: node_id,
                        tag: tag_name,
                        text: String::new(),
                        attrs,
                        parent: None,
                        x: 0.0,
                        y: 0.0,
                        w: 0.0,
                        h: 0.0,
                        bg: None,
                    });
                    rest = &rest[end..];
                }
                None => break,
            }
        }
    }

    fn find_by_selector(&self, selector: &str) -> Option<&MockNode> {
        if let Some(id_str) = selector.strip_prefix('#') {
            self.nodes
                .iter()
                .find(|n| n.attrs.get("id").map(|s| s.as_str()) == Some(id_str))
        } else {
            self.nodes.iter().find(|n| n.tag == selector)
        }
    }
}

impl WebNativeBridge for ServoBridge {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            css_rules: Vec::new(),
            styles: HashMap::new(),
            click_handlers: HashMap::new(),
            form_handlers: HashMap::new(),
            window_open_handler: None,
            click_log: Vec::new(),
            form_log: Vec::new(),
            js_log: Vec::new(),
            js_return: "undefined".to_string(),
            nodes: Vec::new(),
        }
    }

    fn set_html(&mut self, html: &str) {
        self.parse_html(html);
    }

    fn query(&self, selector: &str) -> Option<usize> {
        self.find_by_selector(selector).map(|n| n.id)
    }

    fn query_all(&self, selector: &str) -> Vec<usize> {
        if selector == "*" {
            return self.nodes.iter().map(|n| n.id).collect();
        }
        if let Some(id_str) = selector.strip_prefix('#') {
            self.nodes
                .iter()
                .find(|n| n.attrs.get("id").map(|s| s.as_str()) == Some(id_str))
                .map(|n| vec![n.id])
                .unwrap_or_default()
        } else {
            self.nodes
                .iter()
                .filter(|n| n.tag == selector)
                .map(|n| n.id)
                .collect()
        }
    }

    fn tag_name(&self, node_id: usize) -> Option<String> {
        self.nodes
            .iter()
            .find(|n| n.id == node_id)
            .map(|n| n.tag.clone())
    }

    fn get_attr(&self, node_id: usize, name: &str) -> Option<String> {
        self.nodes
            .iter()
            .find(|n| n.id == node_id)
            .and_then(|n| n.attrs.get(name).cloned())
    }

    fn set_attr(&mut self, node_id: usize, name: &str, value: &str) {
        if let Some(n) = self.nodes.iter_mut().find(|n| n.id == node_id) {
            n.attrs.insert(name.to_string(), value.to_string());
        }
    }

    fn text(&self, node_id: usize) -> Option<String> {
        self.nodes
            .iter()
            .find(|n| n.id == node_id)
            .map(|n| n.text.clone())
    }

    fn parent_node(&self, node_id: usize) -> Option<usize> {
        self.nodes
            .iter()
            .find(|n| n.id == node_id)
            .and_then(|n| n.parent)
    }

    fn get_rect(&self, selector: &str) -> Option<LayoutRect> {
        self.find_by_selector(selector).map(|n| LayoutRect {
            x: n.x,
            y: n.y,
            width: n.w,
            height: n.h,
        })
    }

    fn all_rects(&self) -> Vec<LayoutNode> {
        self.nodes
            .iter()
            .map(|n| LayoutNode {
                dom_node: n.id,
                tag_name: n.tag.clone(),
                x: n.x,
                y: n.y,
                width: n.w,
                height: n.h,
                background: n.bg.clone(),
            })
            .collect()
    }

    fn hit_test(&self, x: f32, y: f32) -> Option<LayoutNode> {
        self.nodes
            .iter()
            .find(|n| x >= n.x && x <= n.x + n.w && y >= n.y && y <= n.y + n.h)
            .map(|n| LayoutNode {
                dom_node: n.id,
                tag_name: n.tag.clone(),
                x: n.x,
                y: n.y,
                width: n.w,
                height: n.h,
                background: n.bg.clone(),
            })
    }

    fn set_css(&mut self, css_text: &str) {
        self.css_rules.push(css_text.to_string());
    }

    fn set_style(&mut self, selector: &str, property: &str, value: &str) {
        self.styles
            .entry(selector.to_string())
            .or_default()
            .push((property.to_string(), value.to_string()));
    }

    fn clear_css(&mut self) {
        self.css_rules.clear();
        self.styles.clear();
    }

    fn eval_js(&mut self, code: &str) -> String {
        self.js_log.push(code.to_string());
        self.js_return.clone()
    }

    fn render(&mut self) -> Vec<u8> {
        vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]
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
        if let Some(sel) = self
            .click_handlers
            .keys()
            .find(|sel| self.query(sel).is_some())
            .cloned()
        {
            if let Some(handler) = self.click_handlers.get_mut(&sel) {
                handler(x, y);
                self.click_log
                    .push(format!("{} @ ({:.0},{:.0})", sel, x, y));
                return true;
            }
        }
        false
    }

    fn handle_form_submit(&mut self, form_selector: &str) {
        if let Some(handler) = self.form_handlers.get_mut(form_selector) {
            handler(HashMap::new());
            self.form_log.push(format!("form: {}", form_selector));
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
    }

    fn viewport(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn navigate(&mut self, _url: &str) -> Result<(), String> {
        Ok(())
    }

    fn current_url(&self) -> String {
        "about:blank".to_string()
    }

    fn http_get(&mut self, _url: &str) -> Result<HttpResponse, String> {
        Err("Network not available in mock implementation".to_string())
    }

    fn http_post(
        &mut self,
        _url: &str,
        _body: &[u8],
        _content_type: &str,
    ) -> Result<HttpResponse, String> {
        Err("Network not available in mock implementation".to_string())
    }

    fn download_file(&mut self, _url: &str, _path: &str) -> Result<u64, String> {
        Err("File operations not available in mock implementation".to_string())
    }

    fn write_file(&mut self, _path: &str, _data: &[u8]) -> Result<(), String> {
        Err("File operations not available in mock implementation".to_string())
    }

    fn read_file(&mut self, _path: &str) -> Result<Vec<u8>, String> {
        Err("File operations not available in mock implementation".to_string())
    }
}
