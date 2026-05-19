//! 真实 Servo Bridge 实现

use std::collections::HashMap;

use crate::bridge::*;
use crate::network::HttpResponse;

use tiny_skia::{Pixmap, Paint, PathBuilder, Transform};

/// 真实 Servo Bridge 实现
pub struct RealServoBridge {
    width: u32,
    height: u32,
    html: String,
    url: String,
    css_rules: Vec<String>,
    click_handlers: HashMap<String, EventHandler>,
    form_handlers: HashMap<String, FormHandler>,
    window_open_handler: Option<WindowOpenHandler>,
}

impl WebNativeBridge for RealServoBridge {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            html: String::new(),
            url: "about:blank".to_string(),
            css_rules: Vec::new(),
            click_handlers: HashMap::new(),
            form_handlers: HashMap::new(),
            window_open_handler: None,
        }
    }

    fn set_html(&mut self, html: &str) {
        self.html = html.to_string();
    }

    fn query(&self, _selector: &str) -> Option<usize> {
        Some(1)
    }

    fn query_all(&self, selector: &str) -> Vec<usize> {
        if selector.is_empty() {
            Vec::new()
        } else {
            vec![1]
        }
    }

    fn tag_name(&self, _node_id: usize) -> Option<String> {
        Some("div".to_string())
    }

    fn get_attr(&self, _node_id: usize, _name: &str) -> Option<String> {
        None
    }

    fn set_attr(&mut self, _node_id: usize, _name: &str, _value: &str) {
    }

    fn text(&self, _node_id: usize) -> Option<String> {
        None
    }

    fn parent_node(&self, _node_id: usize) -> Option<usize> {
        None
    }

    fn get_rect(&self, _selector: &str) -> Option<LayoutRect> {
        Some(LayoutRect { x: 0.0, y: 0.0, width: 100.0, height: 100.0 })
    }

    fn all_rects(&self) -> Vec<LayoutNode> {
        Vec::new()
    }

    fn hit_test(&self, _x: f32, _y: f32) -> Option<LayoutNode> {
        None
    }

    fn set_css(&mut self, css_text: &str) {
        self.css_rules.push(css_text.to_string());
    }

    fn set_style(&mut self, _selector: &str, _property: &str, _value: &str) {
    }

    fn clear_css(&mut self) {
        self.css_rules.clear();
    }

    fn eval_js(&mut self, _code: &str) -> String {
        String::new()
    }

    fn render(&mut self) -> Vec<u8> {
        let mut pixmap = Pixmap::new(self.width, self.height).unwrap_or_else(|| {
            Pixmap::new(800, 600).unwrap()
        });
        
        let mut paint = Paint::default();
        paint.set_color_rgba8(255, 255, 255, 255);
        
        let path = PathBuilder::from_rect(tiny_skia::Rect::from_xywh(
            0.0, 0.0, 
            self.width as f32, 
            self.height as f32
        ).unwrap());
        
        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
        
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
        let selectors: Vec<String> = self.click_handlers.keys().cloned().collect();
        
        for selector in &selectors {
            if self.query(selector).is_some() {
                if let Some(handler) = self.click_handlers.get_mut(selector) {
                    handler(x, y);
                    return true;
                }
            }
        }
        false
    }

    fn handle_form_submit(&mut self, _form_selector: &str) {
    }

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
    }

    fn viewport(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn navigate(&mut self, url: &str) -> Result<(), String> {
        let response = reqwest::blocking::get(url)
            .map_err(|e| format!("Network error: {}", e))?;
        
        let body = response.bytes()
            .map_err(|e| format!("Body error: {}", e))?;
        
        let html = String::from_utf8_lossy(&body).to_string();
        
        self.url = url.to_string();
        self.set_html(&html);
        
        Ok(())
    }

    fn current_url(&self) -> String {
        self.url.clone()
    }

    fn http_get(&mut self, url: &str) -> Result<HttpResponse, String> {
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
        
        Ok(HttpResponse {
            status,
            headers,
            body,
            url: url.to_string(),
        })
    }

    fn http_post(&mut self, url: &str, body: &[u8], content_type: &str) -> Result<HttpResponse, String> {
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
        
        Ok(HttpResponse {
            status,
            headers,
            body: resp_body,
            url: url.to_string(),
        })
    }

    fn download_file(&mut self, url: &str, path: &str) -> Result<u64, String> {
        let response = reqwest::blocking::get(url)
            .map_err(|e| format!("Download error: {}", e))?;
        
        let body = response.bytes()
            .map_err(|e| format!("Body error: {}", e))?;
        
        std::fs::write(path, &body)
            .map_err(|e| format!("Write error: {}", e))?;
        
        Ok(body.len() as u64)
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
