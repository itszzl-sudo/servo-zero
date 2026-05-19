//! 布局树构建和计算

use std::collections::HashMap;
use html_core::dom::{HtmlDocument, NodeId, DomNodeType};
use css_core::cascade::{ComputedStyle, StyleResolver};
use css_core::stylesheet::Stylesheet;
use css_core::Length as CssLength;
use crate::box_model::{LayoutBox, LayoutNode, LayoutRect, DisplayType, PositionType};
use crate::flexbox::FlexboxLayout;

type BoxMap = HashMap<NodeId, LayoutBox>;

/// 布局树
pub struct LayoutTree {
    pub document: HtmlDocument,
    pub style_resolver: StyleResolver,
    boxes: BoxMap,
    viewport_width: f32,
    viewport_height: f32,
}

impl LayoutTree {
    pub fn new(document: HtmlDocument) -> Self {
        Self {
            document,
            style_resolver: StyleResolver::new(),
            boxes: HashMap::new(),
            viewport_width: 800.0,
            viewport_height: 600.0,
        }
    }
    
    /// 创建空的布局树（无 HTML 文档）
    pub fn new_empty() -> Self {
        Self::new(HtmlDocument::new())
    }
    
    /// 获取文档引用
    pub fn document(&self) -> &HtmlDocument {
        &self.document
    }
    
    /// 获取可变文档引用
    pub fn document_mut(&mut self) -> &mut HtmlDocument {
        &mut self.document
    }
    
    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }
    
    pub fn add_stylesheet(&mut self, stylesheet: Stylesheet) {
        self.style_resolver.add_stylesheet(stylesheet);
    }
    
    pub fn add_css(&mut self, css: &str) {
        self.style_resolver.add_css(css);
    }
    
    pub fn set_element_style(&mut self, selector: &str, property: &str, value: &str) {
        self.style_resolver.set_element_style(selector, property, value);
    }
    
    pub fn layout(&mut self) {
        let root_id = match self.document.root_id() {
            Some(id) => id,
            None => return,
        };
        
        self.boxes.clear();
        self.layout_node(root_id, 0.0, 0.0);
    }
    
    fn layout_node(&mut self, node_id: NodeId, _parent_x: f32, _parent_y: f32) {
        let node = match self.document.get_node(node_id) {
            Some(n) => n,
            None => return,
        };
        
        let selector = self.build_selector(node_id);
        let style = self.style_resolver.compute_style(&selector);
        
        let display = self.parse_display(&style);
        
        if display == DisplayType::None {
            return;
        }
        
        let mut layout_node = LayoutNode::new(node_id, node.tag_name_str.clone());
        layout_node.display = display;
        layout_node.position = PositionType::Static;
        layout_node.background = style.background_color;
        layout_node.color = style.color;
        
        // 处理 width
        let width_value: &Option<String> = &style.width;
        if width_value.is_some() {
            let w: &str = width_value.as_ref().unwrap().as_str();
            if let Some(parsed) = CssLength::parse(w) {
                layout_node.width = parsed;
            }
        }
        
        // 处理 height
        let height_value: &Option<String> = &style.height;
        if height_value.is_some() {
            let h: &str = height_value.as_ref().unwrap().as_str();
            if let Some(parsed) = CssLength::parse(h) {
                layout_node.height = parsed;
            }
        }
        
        // 收集子节点（先收集，不做任何可变借用）
        let mut child_ids: Vec<NodeId> = Vec::new();
        for &child_id in &node.children {
            if let Some(child_node) = self.document.get_node(child_id) {
                if child_node.node_type == DomNodeType::Element {
                    child_ids.push(child_id);
                }
            }
        }
        
        // 先递归布局子节点
        for child_id in &child_ids {
            self.layout_node(*child_id, 0.0, 0.0);
        }
        
        // 根据显示类型布局
        match display {
            DisplayType::Flex => {
                let flex_layout = FlexboxLayout::default();
                let mut child_boxes: Vec<LayoutBox> = Vec::new();
                for child_id in &child_ids {
                    if let Some(box_info) = self.boxes.get(child_id) {
                        child_boxes.push(box_info.clone());
                    }
                }
                let mut flex_container_box = LayoutBox::from_node(layout_node.clone(), self.viewport_width, 16.0);
                flex_layout.layout(&mut flex_container_box, &mut child_boxes, self.viewport_width);
                layout_node.rect = flex_container_box.content_box;
            }
            _ => {
                self.layout_block_children(&mut layout_node, &child_ids);
            }
        }
        
        let box_item = LayoutBox::from_node(layout_node.clone(), self.viewport_width, 16.0);
        self.boxes.insert(node_id, box_item);
    }
    
    fn layout_block_children(&mut self, parent: &mut LayoutNode, child_ids: &[NodeId]) {
        let mut y = 0.0_f32;
        
        for child_id in child_ids {
            if let Some(child_box) = self.boxes.get(child_id) {
                let b: &LayoutBox = child_box;
                let mut box_clone: LayoutBox = b.clone();
                box_clone.content_box.x = parent.rect.x;
                box_clone.content_box.y = parent.rect.y + y;
                y += box_clone.content_box.height;
                
                self.boxes.insert(*child_id, box_clone);
            }
        }
        
        if matches!(parent.height, CssLength::Auto) {
            parent.rect.height = y;
        }
    }
    
    fn build_selector(&self, node_id: NodeId) -> String {
        let node = match self.document.get_node(node_id) {
            Some(n) => n,
            None => return String::new(),
        };
        
        let mut selector = String::new();
        
        if let Some(id) = node.attributes.get("id") {
            selector.push('#');
            selector.push_str(id);
        }
        
        if !node.tag_name_str.is_empty() {
            if !selector.is_empty() {
                selector.push(' ');
            }
            selector.push_str(&node.tag_name_str);
        }
        
        if let Some(class_str) = node.attributes.get("class") {
            let class_ref: &str = class_str.as_str();
            for cls in class_ref.split_whitespace() {
                if !selector.is_empty() {
                    selector.push(' ');
                }
                selector.push('.');
                selector.push_str(cls);
            }
        }
        
        let mut current_id = node.parent;
        while let Some(parent_id) = current_id {
            if let Some(parent_node) = self.document.get_node(parent_id) {
                if !parent_node.tag_name_str.is_empty() {
                    if !selector.is_empty() {
                        selector.insert(0, ' ');
                    }
                    selector.insert_str(0, &parent_node.tag_name_str);
                }
                current_id = parent_node.parent;
            } else {
                break;
            }
        }
        
        selector
    }
    
    fn parse_display(&self, style: &ComputedStyle) -> DisplayType {
        match style.display.as_deref() {
            Some("none") => DisplayType::None,
            Some("block") => DisplayType::Block,
            Some("inline") => DisplayType::Inline,
            Some("inline-block") => DisplayType::InlineBlock,
            Some("flex") => DisplayType::Flex,
            Some("grid") => DisplayType::Grid,
            _ => DisplayType::Block,
        }
    }
    
    pub fn all_rects(&self) -> Vec<(NodeId, String, LayoutRect, Option<(u8, u8, u8, u8)>)> {
        let mut results: Vec<(NodeId, String, LayoutRect, Option<(u8, u8, u8, u8)>)> = Vec::new();
        
        for (id, box_info) in self.boxes.iter() {
            let node_id: NodeId = *id;
            if let Some(node) = self.document.get_node(node_id) {
                let rect: LayoutRect = box_info.content_box;
                let bg: Option<(u8, u8, u8, u8)> = box_info.node.background;
                results.push((node_id, node.tag_name_str.clone(), rect, bg));
            }
        }
        
        results
    }
    
    pub fn hit_test(&self, x: f32, y: f32) -> Option<(NodeId, String, LayoutRect)> {
        let mut results: Vec<(NodeId, String, LayoutRect)> = Vec::new();
        
        for (id, box_info) in self.boxes.iter() {
            let node_id: NodeId = *id;
            if box_info.hit_test(x, y) {
                if let Some(node) = self.document.get_node(node_id) {
                    let rect: LayoutRect = box_info.content_box;
                    results.push((node_id, node.tag_name_str.clone(), rect));
                }
            }
        }
        
        results.reverse();
        results.into_iter().next()
    }
    
    pub fn get_rect(&self, selector: &str) -> Option<LayoutRect> {
        self.document.query_selector(selector)
            .and_then(|id: NodeId| -> Option<LayoutRect> {
                self.boxes.get(&id).map(|b| b.content_box)
            })
    }
}
