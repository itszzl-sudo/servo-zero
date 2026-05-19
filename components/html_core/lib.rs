//! HTML 核心解析模块
//! 
//! 基于 html5ever 的独立 HTML 解析器，不依赖 SpiderMonkey

pub mod dom;
pub mod parser;

pub use dom::{DomNode, DomNodeType, HtmlDocument};
pub use parser::HtmlParser;

/// 创建新的空文档
pub fn new_document() -> HtmlDocument {
    HtmlDocument::new()
}
