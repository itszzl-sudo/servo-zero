//! Flexbox 布局算法

use crate::box_model::LayoutBox;

/// Flex 容器方向
#[derive(Debug, Clone, Copy)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

/// Flex 换行方式
#[derive(Debug, Clone, Copy)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

/// Flex 对齐方式
#[derive(Debug, Clone, Copy)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    Baseline,
}

/// 主轴对齐
#[derive(Debug, Clone, Copy)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Flex 项
#[derive(Debug, Clone)]
pub struct FlexItem {
    pub box_ref: LayoutBox,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: f32,
    #[allow(dead_code)]
    pub min_size: f32,
    #[allow(dead_code)]
    pub max_size: f32,
}

impl FlexItem {
    pub fn from_layout_box(r#box: LayoutBox) -> Self {
        Self {
            box_ref: r#box,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: 0.0,
            min_size: 0.0,
            max_size: f32::MAX,
        }
    }
    
    pub fn main_size(&self, container_main_size: f32, total_flex_grow: f32, total_flex_shrink: f32) -> f32 {
        if self.flex_basis > 0.0 {
            return self.flex_basis;
        }
        
        if total_flex_grow > 0.0 && self.box_ref.content_box.width > 0.0 {
            let flex_grow_ratio = self.flex_grow / total_flex_grow;
            return self.box_ref.content_box.width + (container_main_size - self.box_ref.content_box.width) * flex_grow_ratio;
        }
        
        if total_flex_shrink > 0.0 && self.box_ref.content_box.width > 0.0 {
            let flex_shrink_ratio = self.flex_shrink / total_flex_shrink;
            return self.box_ref.content_box.width - self.box_ref.content_box.width * flex_shrink_ratio * 0.5;
        }
        
        self.box_ref.content_box.width
    }
}

/// Flexbox 布局器
pub struct FlexboxLayout {
    pub direction: FlexDirection,
    pub wrap: FlexWrap,
    pub align_items: AlignItems,
    pub justify_content: JustifyContent,
}

impl Default for FlexboxLayout {
    fn default() -> Self {
        Self {
            direction: FlexDirection::Row,
            wrap: FlexWrap::NoWrap,
            align_items: AlignItems::Stretch,
            justify_content: JustifyContent::FlexStart,
        }
    }
}

impl FlexboxLayout {
    pub fn layout(&self, container: &mut LayoutBox, children: &mut [LayoutBox], container_width: f32) {
        if children.is_empty() {
            return;
        }
        
        let is_row = matches!(self.direction, FlexDirection::Row | FlexDirection::RowReverse);
        let main_axis = if is_row { container_width } else { container_width };
        
        let mut items: Vec<FlexItem> = children.iter_mut()
            .map(|b| FlexItem::from_layout_box(b.clone()))
            .collect();
        
        let total_flex_grow: f32 = items.iter().map(|i| i.flex_grow).sum();
        let total_flex_shrink: f32 = items.iter().map(|i| i.flex_shrink).sum();
        
        let mut current_pos = 0.0_f32;
        
        for (i, item) in items.iter_mut().enumerate() {
            let item_main_size = item.main_size(main_axis, total_flex_grow, total_flex_shrink);
            
            let cross_size = if is_row {
                item.box_ref.content_box.height
            } else {
                item.box_ref.content_box.width
            };
            
            let cross_pos = match self.align_items {
                AlignItems::FlexStart => 0.0,
                AlignItems::FlexEnd => container_width - cross_size,
                AlignItems::Center => (container_width - cross_size) / 2.0,
                AlignItems::Stretch => 0.0,
                AlignItems::Baseline => 0.0,
            };
            
            if is_row {
                item.box_ref.content_box.x = container.content_box.x + current_pos;
                item.box_ref.content_box.y = container.content_box.y + cross_pos;
                item.box_ref.content_box.width = item_main_size;
                current_pos += item_main_size;
            } else {
                item.box_ref.content_box.x = container.content_box.x + cross_pos;
                item.box_ref.content_box.y = container.content_box.y + current_pos;
                item.box_ref.content_box.height = item_main_size;
                current_pos += item_main_size;
            }
            
            children[i].content_box = item.box_ref.content_box.clone();
        }
        
        if is_row {
            container.content_box.height = items.iter()
                .map(|i| i.box_ref.content_box.height)
                .fold(0.0_f32, |a, b| a.max(b));
        } else {
            container.content_box.width = items.iter()
                .map(|i| i.box_ref.content_box.width)
                .fold(0.0_f32, |a, b| a.max(b));
        }
    }
}
