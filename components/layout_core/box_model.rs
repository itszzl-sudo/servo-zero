//! CSS 盒模型定义

use css_core::Length as CssLength;

/// 布局矩形
#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LayoutRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
    
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width &&
        y >= self.y && y <= self.y + self.height
    }
}

/// 布局节点
#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub node_id: usize,
    pub tag_name: String,
    pub display: DisplayType,
    pub position: PositionType,
    pub rect: LayoutRect,
    pub background: Option<(u8, u8, u8, u8)>,
    pub color: Option<(u8, u8, u8, u8)>,
    pub width: CssLength,
    pub height: CssLength,
    pub margin: SideValues,
    pub padding: SideValues,
    pub border: SideValues,
    pub children: Vec<usize>,
}

impl LayoutNode {
    pub fn new(node_id: usize, tag_name: String) -> Self {
        Self {
            node_id,
            tag_name,
            display: DisplayType::Block,
            position: PositionType::Static,
            rect: LayoutRect::default(),
            background: None,
            color: None,
            width: CssLength::Auto,
            height: CssLength::Auto,
            margin: SideValues::default(),
            padding: SideValues::default(),
            border: SideValues::default(),
            children: Vec::new(),
        }
    }
}

/// 显示类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplayType {
    None,
    Block,
    Inline,
    InlineBlock,
    Flex,
    Grid,
}

impl Default for DisplayType {
    fn default() -> Self {
        DisplayType::Block
    }
}

/// 位置类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionType {
    Static,
    Relative,
    Absolute,
    Fixed,
}

impl Default for PositionType {
    fn default() -> Self {
        PositionType::Static
    }
}

/// 四边值（上下左右）
#[derive(Debug, Clone, Default)]
pub struct SideValues {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl SideValues {
    pub fn uniform(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
    
    pub fn parse(value: &str) -> Self {
        let parts: Vec<&str> = value.split_whitespace().collect();
        match parts.len() {
            1 => {
                let v = parts[0].parse::<f32>().unwrap_or(0.0);
                Self::uniform(v)
            }
            2 => {
                let v1 = parts[0].parse::<f32>().unwrap_or(0.0);
                let v2 = parts[1].parse::<f32>().unwrap_or(0.0);
                Self {
                    top: v1,
                    right: v2,
                    bottom: v1,
                    left: v2,
                }
            }
            3 => {
                let v1 = parts[0].parse::<f32>().unwrap_or(0.0);
                let v2 = parts[1].parse::<f32>().unwrap_or(0.0);
                let v3 = parts[2].parse::<f32>().unwrap_or(0.0);
                Self {
                    top: v1,
                    right: v2,
                    bottom: v3,
                    left: v2,
                }
            }
            4 => {
                Self {
                    top: parts[0].parse::<f32>().unwrap_or(0.0),
                    right: parts[1].parse::<f32>().unwrap_or(0.0),
                    bottom: parts[2].parse::<f32>().unwrap_or(0.0),
                    left: parts[3].parse::<f32>().unwrap_or(0.0),
                }
            }
            _ => Self::default(),
        }
    }
}

/// 布局盒
#[derive(Debug, Clone)]
pub struct LayoutBox {
    pub node: LayoutNode,
    pub content_box: LayoutRect,
    pub padding_box: LayoutRect,
    pub border_box: LayoutRect,
    pub margin_box: LayoutRect,
}

impl LayoutBox {
    pub fn from_node(node: LayoutNode, parent_width: f32, font_size: f32) -> Self {
        let width = node.width.to_px(parent_width, font_size);
        let height = node.height.to_px(parent_width, font_size);
        
        let margin = &node.margin;
        let padding = &node.padding;
        let border = &node.border;
        
        let content_box = LayoutRect::new(0.0, 0.0, width.max(0.0), height.max(0.0));
        
        let padding_box = LayoutRect::new(
            padding.left,
            padding.top,
            content_box.width + padding.left + padding.right,
            content_box.height + padding.top + padding.bottom,
        );
        
        let border_box = LayoutRect::new(
            padding_box.x - border.left,
            padding_box.y - border.top,
            padding_box.width + border.left + border.right,
            padding_box.height + border.top + border.bottom,
        );
        
        let margin_box = LayoutRect::new(
            border_box.x - margin.left,
            border_box.y - margin.top,
            border_box.width + margin.left + margin.right,
            border_box.height + margin.top + margin.bottom,
        );
        
        Self {
            node,
            content_box,
            padding_box,
            border_box,
            margin_box,
        }
    }
    
    pub fn hit_test(&self, x: f32, y: f32) -> bool {
        self.border_box.contains_point(x, y)
    }
}
