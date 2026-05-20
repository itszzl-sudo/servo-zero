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
    /// CSS font-size 解析后的像素值
    pub font_size: Option<f32>,
    /// CSS font-weight
    pub font_weight: Option<String>,
    /// CSS font-family
    pub font_family: Option<String>,
    /// CSS text-align
    pub text_align: Option<String>,
    /// CSS border-width（px）
    pub border_width: Option<f32>,
    /// CSS border-color
    pub border_color: Option<(u8, u8, u8, u8)>,
    /// 定位偏移（用于 relative/absolute/fixed）
    pub position_top: Option<f32>,
    pub position_right: Option<f32>,
    pub position_bottom: Option<f32>,
    pub position_left: Option<f32>,
    /// CSS transform 变换矩阵 [a, b, c, d, tx, ty]
    pub transform_matrix: Option<[f32; 6]>,
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
            font_size: None,
            font_weight: None,
            font_family: None,
            text_align: None,
            border_width: None,
            border_color: None,
            position_top: None,
            position_right: None,
            position_bottom: None,
            position_left: None,
            transform_matrix: None,
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
                // 使用 CssLength::parse 解析带单位的值
                let v = CssLength::parse(parts[0])
                    .map(|l| l.to_px(0.0, 16.0, 800.0, 600.0))
                    .unwrap_or(parts[0].parse::<f32>().unwrap_or(0.0));
                Self::uniform(v)
            }
            2 => {
                let v1 = CssLength::parse(parts[0])
                    .map(|l| l.to_px(0.0, 16.0, 800.0, 600.0))
                    .unwrap_or(parts[0].parse::<f32>().unwrap_or(0.0));
                let v2 = CssLength::parse(parts[1])
                    .map(|l| l.to_px(0.0, 16.0, 800.0, 600.0))
                    .unwrap_or(parts[1].parse::<f32>().unwrap_or(0.0));
                Self {
                    top: v1,
                    right: v2,
                    bottom: v1,
                    left: v2,
                }
            }
            3 => {
                let v1 = CssLength::parse(parts[0])
                    .map(|l| l.to_px(0.0, 16.0, 800.0, 600.0))
                    .unwrap_or(parts[0].parse::<f32>().unwrap_or(0.0));
                let v2 = CssLength::parse(parts[1])
                    .map(|l| l.to_px(0.0, 16.0, 800.0, 600.0))
                    .unwrap_or(parts[1].parse::<f32>().unwrap_or(0.0));
                let v3 = CssLength::parse(parts[2])
                    .map(|l| l.to_px(0.0, 16.0, 800.0, 600.0))
                    .unwrap_or(parts[2].parse::<f32>().unwrap_or(0.0));
                Self {
                    top: v1,
                    right: v2,
                    bottom: v3,
                    left: v2,
                }
            }
            4 => {
                Self {
                    top: CssLength::parse(parts[0])
                        .map(|l| l.to_px(0.0, 16.0, 800.0, 600.0))
                        .unwrap_or(parts[0].parse::<f32>().unwrap_or(0.0)),
                    right: CssLength::parse(parts[1])
                        .map(|l| l.to_px(0.0, 16.0, 800.0, 600.0))
                        .unwrap_or(parts[1].parse::<f32>().unwrap_or(0.0)),
                    bottom: CssLength::parse(parts[2])
                        .map(|l| l.to_px(0.0, 16.0, 800.0, 600.0))
                        .unwrap_or(parts[2].parse::<f32>().unwrap_or(0.0)),
                    left: CssLength::parse(parts[3])
                        .map(|l| l.to_px(0.0, 16.0, 800.0, 600.0))
                        .unwrap_or(parts[3].parse::<f32>().unwrap_or(0.0)),
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
        let width = node.width.to_px(parent_width, font_size, 800.0, 600.0);
        let height = node.height.to_px(parent_width, font_size, 800.0, 600.0);
        
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
