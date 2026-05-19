//! 网络请求相关类型定义

use std::collections::HashMap;

/// HTTP 响应结构体
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// HTTP 状态码
    pub status: u16,
    /// 响应头
    pub headers: HashMap<String, String>,
    /// 响应体
    pub body: Vec<u8>,
    /// 响应URL（可能与请求URL不同，处理重定向后）
    pub url: String,
}

impl HttpResponse {
    /// 创建新的HTTP响应
    pub fn new(status: u16, body: Vec<u8>) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body,
            url: String::new(),
        }
    }

    /// 创建成功响应（200 OK）
    pub fn ok(body: Vec<u8>) -> Self {
        Self::new(200, body)
    }

    /// 创建失败响应（500 Internal Server Error）
    pub fn error(body: Vec<u8>) -> Self {
        Self::new(500, body)
    }

    /// 获取响应体的字符串表示
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }

    /// 设置响应头
    pub fn set_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_string(), value.to_string());
    }

    /// 获取响应头
    pub fn get_header(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }
}
