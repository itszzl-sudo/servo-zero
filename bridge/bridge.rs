//! WebNativeBridge 接口定义

use std::collections::HashMap;

/// RGBA 颜色
#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const RED: Color = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const TRANSPARENT: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        let len = hex.len();
        let parse_hex = |s: &str| u8::from_str_radix(s, 16).unwrap_or(0);
        match len {
            3 => Self {
                r: parse_hex(&hex[0..1]) * 17,
                g: parse_hex(&hex[1..2]) * 17,
                b: parse_hex(&hex[2..3]) * 17,
                a: 255,
            },
            6 => Self {
                r: parse_hex(&hex[0..2]),
                g: parse_hex(&hex[2..4]),
                b: parse_hex(&hex[4..6]),
                a: 255,
            },
            8 => Self {
                r: parse_hex(&hex[0..2]),
                g: parse_hex(&hex[2..4]),
                b: parse_hex(&hex[4..6]),
                a: parse_hex(&hex[6..8]),
            },
            _ => Self {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
        }
    }
}

/// 元素在页面上的布局矩形
#[derive(Debug, Clone, Copy, serde::Deserialize)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// 单个布局节点的详细信息
#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub dom_node: usize,
    pub tag_name: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub background: Option<Color>,
}

/// CSS 声明
#[derive(Debug, Clone)]
pub struct Declaration {
    pub property: String,
    pub value: String,
}

/// 点击事件处理器：接收点击坐标 (x, y)
pub type EventHandler = Box<dyn FnMut(f32, f32) + Send>;

/// 表单提交处理器：接收字段名 → 值的映射
pub type FormHandler = Box<dyn FnMut(HashMap<String, String>) + Send>;

/// window.open 处理器：接收 URL，返回是否处理
pub type WindowOpenHandler = Box<dyn FnMut(&str) -> bool + Send>;

/// Web → Native 桥接器接口
///
/// 所有方法都有默认实现或可直接调用，外部项目实现此 trait 即可。
pub trait WebNativeBridge {
    /// 创建桥接器
    fn new(width: u32, height: u32) -> Self
    where
        Self: Sized;

    // ── DOM 读写 ──

    /// 设置页面 HTML
    fn set_html(&mut self, html: &str);

    /// 按 CSS 选择器查找第一个元素，返回 DOM 节点 ID
    fn query(&self, selector: &str) -> Option<usize>;

    /// 按 CSS 选择器查找所有匹配元素
    fn query_all(&self, selector: &str) -> Vec<usize>;

    /// 获取元素标签名
    fn tag_name(&self, node_id: usize) -> Option<String>;

    /// 获取元素属性
    fn get_attr(&self, node_id: usize, name: &str) -> Option<String>;

    /// 设置元素属性
    fn set_attr(&mut self, node_id: usize, name: &str, value: &str);

    /// 获取元素文本内容
    fn text(&self, node_id: usize) -> Option<String>;

    /// 获取父节点 ID
    fn parent_node(&self, node_id: usize) -> Option<usize>;

    /// 按选择器获取元素文本
    fn query_text(&self, selector: &str) -> Option<String> {
        let id = self.query(selector)?;
        self.text(id)
    }

    // ── 布局 ──

    /// 获取元素在页面上的位置
    fn get_rect(&self, selector: &str) -> Option<LayoutRect>;

    /// 获取所有布局节点
    fn all_rects(&self) -> Vec<LayoutNode>;

    /// 点击测试
    fn hit_test(&self, x: f32, y: f32) -> Option<LayoutNode>;

    // ── CSS 操作 ──

    /// 添加 CSS 规则
    fn set_css(&mut self, css_text: &str);

    /// 设置元素内联样式
    fn set_style(&mut self, selector: &str, property: &str, value: &str);

    /// 清除自定义 CSS
    fn clear_css(&mut self);

    // ── JS 执行 ──

    /// 执行 JavaScript 代码
    fn eval_js(&mut self, code: &str) -> String;

    // ── 渲染 ──

    /// 渲染当前页面，返回 PNG 字节
    fn render(&mut self) -> Vec<u8>;

    // ── 事件绑定 ──

    /// 绑定点击事件
    fn on_click(&mut self, selector: &str, handler: EventHandler);

    /// 绑定表单提交事件
    fn on_form_submit(&mut self, selector: &str, handler: FormHandler);

    /// 绑定 window.open 事件（用于附件下载）
    fn on_window_open(&mut self, handler: WindowOpenHandler);

    /// 处理鼠标点击（事件冒泡）
    fn handle_click(&mut self, x: f32, y: f32) -> bool;

    /// 处理表单提交
    fn handle_form_submit(&mut self, form_selector: &str);

    /// 处理 window.open 调用（返回是否已处理）
    fn handle_window_open(&mut self, url: &str) -> bool;

    // ── 工具 ──

    /// 设置视口尺寸
    fn set_viewport(&mut self, width: u32, height: u32);

    /// 获取视口尺寸
    fn viewport(&self) -> (u32, u32);

    // ── 网络请求 ──

    /// 导航到 URL
    fn navigate(&mut self, url: &str) -> Result<(), String>;

    /// 获取当前 URL
    fn current_url(&self) -> String;

    /// 发送 HTTP GET 请求
    fn http_get(&mut self, url: &str) -> Result<crate::network::HttpResponse, String>;

    /// 发送 HTTP POST 请求
    fn http_post(
        &mut self,
        url: &str,
        body: &[u8],
        content_type: &str,
    ) -> Result<crate::network::HttpResponse, String>;

    // ── 文件操作 ──

    /// 下载文件并保存到本地
    fn download_file(&mut self, url: &str, path: &str) -> Result<u64, String>;

    /// 写入文件
    fn write_file(&mut self, path: &str, data: &[u8]) -> Result<(), String>;

    /// 读取文件
    fn read_file(&mut self, path: &str) -> Result<Vec<u8>, String>;
}
