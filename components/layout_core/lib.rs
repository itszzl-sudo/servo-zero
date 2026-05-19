//! 布局核心模块
//! 
//! 提供独立的 CSS 布局引擎，支持盒模型和 Flexbox，不依赖 SpiderMonkey

mod box_model;
mod flexbox;
mod tree;

pub use box_model::{LayoutBox, LayoutRect, LayoutNode, DisplayType, PositionType};
pub use flexbox::FlexboxLayout;
pub use tree::LayoutTree;

/// 布局计算结果
#[derive(Debug, Clone)]
pub struct LayoutResult {
    pub node_id: usize,
    pub tag_name: String,
    pub rect: LayoutRect,
    pub background: Option<(u8, u8, u8, u8)>,
    pub children: Vec<LayoutResult>,
}
