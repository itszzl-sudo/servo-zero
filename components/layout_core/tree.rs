//! 布局树构建和计算
//!
//! 使用 taffy 库进行实际布局计算，与 servo/components/layout 同款引擎。

use std::collections::HashMap;
use taffy::prelude::*;
use taffy::{Overflow, Point};
use html_core::dom::{HtmlDocument, NodeId as DomNodeId, DomNodeType};
use css_core::cascade::{ComputedStyle, StyleResolver};
use css_core::stylesheet::Stylesheet;
use css_core::Length as CssLength;
use crate::box_model::{LayoutBox, LayoutNode, LayoutRect, DisplayType, PositionType};

type BoxMap = HashMap<DomNodeId, LayoutBox>;

/// 字体测量函数类型
pub type FontMeasureFunc = fn(text: &str, font_size: f32, max_width: f32) -> (f32, f32);

/// 布局树
pub struct LayoutTree {
    pub document: HtmlDocument,
    pub style_resolver: StyleResolver,
    boxes: BoxMap,
    viewport_width: f32,
    viewport_height: f32,
    font_measure: Option<FontMeasureFunc>,
}

impl LayoutTree {
    pub fn new(document: HtmlDocument) -> Self {
        Self {
            document,
            style_resolver: StyleResolver::new(),
            boxes: HashMap::new(),
            viewport_width: 800.0,
            viewport_height: 600.0,
            font_measure: None,
        }
    }

    pub fn new_empty() -> Self {
        Self::new(HtmlDocument::new())
    }

    pub fn document(&self) -> &HtmlDocument {
        &self.document
    }

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

    /// 执行布局计算（使用 taffy 引擎）
    pub fn layout(&mut self) {
        let root_id = match self.find_layout_root() {
            Some(id) => id,
            None => {
                log::warn!("layout: 没有根节点");
                return;
            }
        };

        log::info!("layout: 开始布局，根节点 ID: {}", root_id);

        let mut taffy: TaffyTree<()> = TaffyTree::new();
        let mut dom_to_taffy: HashMap<DomNodeId, NodeId> = HashMap::new();
        let mut computed_styles: HashMap<DomNodeId, ComputedStyle> = HashMap::new();

        // 1. 从 DOM 构建 taffy 布局树
        let taffy_root = match self.build_taffy_node(
            &mut taffy, root_id, &mut dom_to_taffy, &mut computed_styles, "",
        ) {
            Some(n) => n,
            None => {
                log::warn!("layout: 无法构建 taffy 树（根节点不可见？）");
                return;
            }
        };

        // 2. taffy 计算布局
        if let Err(e) = taffy.compute_layout(
            taffy_root,
            Size {
                width: AvailableSpace::Definite(self.viewport_width),
                height: AvailableSpace::Definite(self.viewport_height),
            },
        ) {
            log::warn!("layout: taffy compute_layout 失败: {:?}", e);
            return;
        }

        // 3. 提取布局结果到 self.boxes
        self.boxes.clear();
        self.extract_layout(
            &taffy, taffy_root, root_id, 0.0, 0.0, &dom_to_taffy, &computed_styles,
        );

        log::info!("layout: 布局完成，生成的 box 数量: {}", self.boxes.len());
    }

    // ── 内部实现 ──────────────────────────────────────────────────────────────

    /// 找到布局起始根节点（跳过 Document 虚节点，从第一个 Element 开始）
    fn find_layout_root(&self) -> Option<DomNodeId> {
        let root_id = self.document.root_id()?;
        let root_type = self.document.get_node(root_id)?.node_type.clone();

        if root_type == html_core::dom::DomNodeType::Document {
            let children = self.document.get_node(root_id)?.children.clone();
            for child_id in children {
                if let Some(child) = self.document.get_node(child_id) {
                    if child.node_type == html_core::dom::DomNodeType::Element {
                        return Some(child_id);
                    }
                }
            }
            None
        } else {
            Some(root_id)
        }
    }

    /// 递归构建 taffy 布局树，返回对应的 taffy NodeId
    fn build_taffy_node(
        &self,
        taffy: &mut TaffyTree<()>,
        node_id: DomNodeId,
        dom_to_taffy: &mut HashMap<DomNodeId, NodeId>,
        computed_styles: &mut HashMap<DomNodeId, ComputedStyle>,
        ancestor_path: &str,
    ) -> Option<NodeId> {
        // 取出必要信息后立即释放对 document 的借用，避免递归时冲突
        let (node_type, tag, child_ids) = {
            let node = self.document.get_node(node_id)?;
            (node.node_type.clone(), node.tag_name_str.clone(), node.children.clone())
        };

        // 只处理元素节点
        if node_type != html_core::dom::DomNodeType::Element {
            return None;
        }

        // 跳过不渲染的标签
        if matches!(
            tag.as_str(),
            "head" | "script" | "style" | "meta" | "link" | "title" | "noscript"
        ) {
            return None;
        }

        let selector = self.build_selector(node_id);
        let local_for_path = self.build_local_selector_for_path(node_id);
        let id_selector = if selector.starts_with('#') { Some(selector.as_str()) } else { None };
        let style = self.style_resolver.compute_style_with_path(
            &local_for_path, id_selector, ancestor_path, Some(node_id),
        );

        // display:none 不参与布局
        if style.display.as_deref() == Some("none") {
            return None;
        }

        let taffy_style = self.css_to_taffy_style(&style);

        // 构建子节点的祖先路径
        let child_ancestor_path = if ancestor_path.is_empty() {
            local_for_path.clone()
        } else {
            format!("{} {}", ancestor_path, &local_for_path)
        };

        // 递归构建可见子节点
        let mut taffy_children: Vec<NodeId> = Vec::new();
        for &child_id in &child_ids {
            if let Some(child_taffy) =
                self.build_taffy_node(taffy, child_id, dom_to_taffy, computed_styles, &child_ancestor_path)
            {
                taffy_children.push(child_taffy);
            }
        }

        // 保存计算样式供 extract_layout 使用
        computed_styles.insert(node_id, style);

        let taffy_node = if taffy_children.is_empty() {
            taffy.new_leaf(taffy_style).ok()?
        } else {
            taffy.new_with_children(taffy_style, &taffy_children).ok()?
        };

        dom_to_taffy.insert(node_id, taffy_node);
        Some(taffy_node)
    }

    /// 将 CSS ComputedStyle 映射为 taffy::Style
    fn css_to_taffy_style(&self, style: &ComputedStyle) -> Style {
        let display = match style.display.as_deref() {
            Some("none") => Display::None,
            Some("flex") | Some("inline-flex") => Display::Flex,
            Some("grid") | Some("inline-grid") => Display::Grid,
            _ => Display::Block,
        };

        // Dimension（width / height）
        let parse_dim = |s: Option<&str>| -> Dimension {
            s.and_then(|v| CssLength::parse(v))
                .map(|l| match l {
                    CssLength::Px(v) => Dimension::from_length(v),
                    CssLength::Percent(v) => Dimension::from_percent(v / 100.0),
                    _ => Dimension::AUTO,
                })
                .unwrap_or(Dimension::AUTO)
        };

        // LengthPercentageAuto（margin）
        let parse_margin = |s: Option<&str>| -> LengthPercentageAuto {
            s.and_then(|v| CssLength::parse(v))
                .map(|l| match l {
                    CssLength::Px(v) => LengthPercentageAuto::from_length(v),
                    CssLength::Percent(v) => LengthPercentageAuto::from_percent(v / 100.0),
                    _ => LengthPercentageAuto::AUTO,
                })
                .unwrap_or(LengthPercentageAuto::AUTO)
        };

        // LengthPercentage（padding / border）
        let parse_padding = |s: Option<&str>| -> LengthPercentage {
            s.and_then(|v| CssLength::parse(v))
                .map(|l| match l {
                    CssLength::Px(v) => LengthPercentage::from_length(v),
                    CssLength::Percent(v) => LengthPercentage::from_percent(v / 100.0),
                    _ => LengthPercentage::from_length(0.0),
                })
                .unwrap_or(LengthPercentage::from_length(0.0))
        };

        // Flexbox 属性（存储在 ComputedStyle::other 中）
        let flex_direction = match style.other.get("flex-direction").map(|s| s.as_str()) {
            Some("column") => FlexDirection::Column,
            Some("column-reverse") => FlexDirection::ColumnReverse,
            Some("row-reverse") => FlexDirection::RowReverse,
            _ => FlexDirection::Row,
        };

        let flex_wrap = match style.other.get("flex-wrap").map(|s| s.as_str()) {
            Some("wrap") => FlexWrap::Wrap,
            Some("wrap-reverse") => FlexWrap::WrapReverse,
            _ => FlexWrap::NoWrap,
        };

        let justify_content = match style.other.get("justify-content").map(|s| s.as_str()) {
            Some("flex-end") | Some("end") => Some(JustifyContent::FlexEnd),
            Some("center") => Some(JustifyContent::Center),
            Some("space-between") => Some(JustifyContent::SpaceBetween),
            Some("space-around") => Some(JustifyContent::SpaceAround),
            Some("space-evenly") => Some(JustifyContent::SpaceEvenly),
            _ => None,
        };

        let align_items = match style.other.get("align-items").map(|s| s.as_str()) {
            Some("flex-start") | Some("start") => Some(AlignItems::FlexStart),
            Some("flex-end") | Some("end") => Some(AlignItems::FlexEnd),
            Some("center") => Some(AlignItems::Center),
            Some("baseline") => Some(AlignItems::Baseline),
            _ => None,
        };

        // Flex 子项属性（应用到子元素自身）
        let flex_grow = style.other.get("flex-grow")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0);

        let flex_shrink = style.other.get("flex-shrink")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(1.0);

        let flex_basis = style.other.get("flex-basis")
            .and_then(|s| CssLength::parse(s))
            .map(|l| match l {
                CssLength::Px(v) => Dimension::from_length(v),
                CssLength::Percent(v) => Dimension::from_percent(v / 100.0),
                _ => Dimension::AUTO,
            })
            .unwrap_or(Dimension::AUTO);

        // flex 简写：flex: <grow> <shrink> <basis>
        let (flex_grow, flex_shrink, flex_basis) = if let Some(flex_val) = style.other.get("flex") {
            let parts: Vec<&str> = flex_val.split_whitespace().collect();
            let fg = parts.first().and_then(|s| s.parse::<f32>().ok()).unwrap_or(1.0);
            let fs = parts.get(1).and_then(|s| s.parse::<f32>().ok()).unwrap_or(1.0);
            let fb = parts.get(2)
                .and_then(|s| CssLength::parse(s))
                .map(|l| match l {
                    CssLength::Px(v) => Dimension::from_length(v),
                    CssLength::Percent(v) => Dimension::from_percent(v / 100.0),
                    _ => Dimension::AUTO,
                })
                .unwrap_or(Dimension::AUTO);
            (fg, fs, fb)
        } else {
            (flex_grow, flex_shrink, flex_basis)
        };

        // gap / row-gap / column-gap
        let parse_gap = |key: &str| -> LengthPercentage {
            style.other.get(key)
                .and_then(|s| CssLength::parse(s))
                .map(|l| match l {
                    CssLength::Px(v) => LengthPercentage::from_length(v),
                    CssLength::Percent(v) => LengthPercentage::from_percent(v / 100.0),
                    _ => LengthPercentage::from_length(0.0),
                })
                .unwrap_or(LengthPercentage::from_length(0.0))
        };
        let gap_size = Size {
            width: parse_gap("column-gap"),
            height: parse_gap("row-gap"),
        };

        // Position (static/relative → Relative, absolute/fixed → Absolute)
        let position = match style.position.as_deref() {
            Some("absolute") | Some("fixed") => Position::Absolute,
            _ => Position::Relative,
        };

        // Inset: top / right / bottom / left
        let inset = Rect {
            top: parse_margin(style.top.as_deref()),
            right: parse_margin(style.right.as_deref()),
            bottom: parse_margin(style.bottom.as_deref()),
            left: parse_margin(style.left.as_deref()),
        };

        // Max/Min size
        let max_size = Size {
            width: parse_dim(style.other.get("max-width").map(|s| s.as_str())),
            height: parse_dim(style.other.get("max-height").map(|s| s.as_str())),
        };
        let min_size = Size {
            width: parse_dim(style.other.get("min-width").map(|s| s.as_str())),
            height: parse_dim(style.other.get("min-height").map(|s| s.as_str())),
        };

        // Overflow
        let parse_ov = |key_sp: &str, key_gen: &str| -> Overflow {
            let v = style.other.get(key_sp).or_else(|| style.other.get(key_gen));
            match v.map(|s| s.as_str()) {
                Some("hidden") => Overflow::Hidden,
                Some("scroll") => Overflow::Scroll,
                Some("clip") => Overflow::Clip,
                _ => Overflow::Visible,
            }
        };
        let overflow = Point {
            x: parse_ov("overflow-x", "overflow"),
            y: parse_ov("overflow-y", "overflow"),
        };

        Style {
            display,
            position,
            size: Size {
                width: parse_dim(style.width.as_deref()),
                height: parse_dim(style.height.as_deref()),
            },
            min_size,
            max_size,
            inset,
            margin: Rect {
                top: parse_margin(style.margin_top.as_deref()),
                bottom: parse_margin(style.margin_bottom.as_deref()),
                left: parse_margin(style.margin_left.as_deref()),
                right: parse_margin(style.margin_right.as_deref()),
            },
            padding: Rect {
                top: parse_padding(style.padding_top.as_deref()),
                bottom: parse_padding(style.padding_bottom.as_deref()),
                left: parse_padding(style.padding_left.as_deref()),
                right: parse_padding(style.padding_right.as_deref()),
            },
            flex_direction,
            flex_wrap,
            justify_content,
            align_items,
            flex_grow,
            flex_shrink,
            flex_basis,
            gap: gap_size,
            overflow,
            ..Default::default()
        }
    }

    /// 从 taffy 布局结果提取坐标并填充 self.boxes
    fn extract_layout(
        &mut self,
        taffy: &TaffyTree<()>,
        taffy_node: NodeId,
        dom_node_id: DomNodeId,
        parent_x: f32,
        parent_y: f32,
        dom_to_taffy: &HashMap<DomNodeId, NodeId>,
        computed_styles: &HashMap<DomNodeId, ComputedStyle>,
    ) {
        let layout = match taffy.layout(taffy_node) {
            Ok(l) => l,
            Err(_) => return,
        };

        // taffy 给出的是相对父节点的偏移量
        let x = parent_x + layout.location.x;
        let y = parent_y + layout.location.y;
        let w = layout.size.width;
        let h = layout.size.height;

        // 取出 DOM 数据并立即释放借用
        let (tag, child_ids) = {
            match self.document.get_node(dom_node_id) {
                Some(n) => (n.tag_name_str.clone(), n.children.clone()),
                None => return,
            }
        };

        let style = computed_styles.get(&dom_node_id).cloned().unwrap_or_default();

        let mut layout_node = LayoutNode::new(dom_node_id, tag);
        layout_node.display = match style.display.as_deref() {
            Some("none") => DisplayType::None,
            Some("flex") | Some("inline-flex") => DisplayType::Flex,
            Some("grid") | Some("inline-grid") => DisplayType::Grid,
            Some("inline") => DisplayType::Inline,
            Some("inline-block") => DisplayType::InlineBlock,
            _ => DisplayType::Block,
        };
        // 正确映射 position 类型
        layout_node.position = match style.position.as_deref() {
            Some("absolute") => PositionType::Absolute,
            Some("fixed") => PositionType::Fixed,
            Some("relative") => PositionType::Relative,
            _ => PositionType::Static,
        };
        layout_node.background = style.background_color;
        layout_node.color = style.color;
        layout_node.font_size = style.font_size.as_deref().and_then(parse_css_px);
        layout_node.font_weight = style.font_weight.clone();
        layout_node.font_family = style.font_family.clone();
        layout_node.text_align = style.text_align.clone();
        layout_node.border_width = style.border_width;
        layout_node.border_color = style.border_color;

        // 计算四边偏移（padding / border / margin）
        let pw = style.padding_top.as_deref().and_then(parse_css_px).unwrap_or(0.0)
            + style.padding_bottom.as_deref().and_then(parse_css_px).unwrap_or(0.0);
        let ph = style.padding_left.as_deref().and_then(parse_css_px).unwrap_or(0.0)
            + style.padding_right.as_deref().and_then(parse_css_px).unwrap_or(0.0);
        let bw = style.border_width.unwrap_or(0.0) * 2.0; // 左右边框
        let bh = style.border_width.unwrap_or(0.0) * 2.0; // 上下边框
        let mw = style.margin_left.as_deref().and_then(parse_css_px).unwrap_or(0.0)
            + style.margin_right.as_deref().and_then(parse_css_px).unwrap_or(0.0);
        let mh = style.margin_top.as_deref().and_then(parse_css_px).unwrap_or(0.0)
            + style.margin_bottom.as_deref().and_then(parse_css_px).unwrap_or(0.0);

        let content_box = LayoutRect { x, y, width: w, height: h };
        let padding_box = LayoutRect {
            x: x - style.padding_left.as_deref().and_then(parse_css_px).unwrap_or(0.0),
            y: y - style.padding_top.as_deref().and_then(parse_css_px).unwrap_or(0.0),
            width: w + pw,
            height: h + ph,
        };
        let border_box = LayoutRect {
            x: padding_box.x - style.border_width.unwrap_or(0.0),
            y: padding_box.y - style.border_width.unwrap_or(0.0),
            width: padding_box.width + bw,
            height: padding_box.height + bh,
        };
        let margin_box = LayoutRect {
            x: border_box.x - style.margin_left.as_deref().and_then(parse_css_px).unwrap_or(0.0),
            y: border_box.y - style.margin_top.as_deref().and_then(parse_css_px).unwrap_or(0.0),
            width: border_box.width + mw,
            height: border_box.height + mh,
        };

        self.boxes.insert(
            dom_node_id,
            LayoutBox {
                node: layout_node,
                content_box,
                padding_box,
                border_box,
                margin_box,
            },
        );

        // 递归处理在 taffy 树中存在的 DOM 子节点
        for &child_dom_id in &child_ids {
            if let Some(&child_taffy_id) = dom_to_taffy.get(&child_dom_id) {
                self.extract_layout(
                    taffy,
                    child_taffy_id,
                    child_dom_id,
                    x,
                    y,
                    dom_to_taffy,
                    computed_styles,
                );
            }
        }
    }

    /// 从 DOM 节点构建 CSS 选择器字符串
    fn build_selector(&self, node_id: DomNodeId) -> String {
        let node = match self.document.get_node(node_id) {
            Some(n) => n,
            None => return String::new(),
        };

        // 优先 ID 选择器
        if let Some(id) = node.get_attr("id") {
            return format!("#{}", id);
        }

        let mut selector = node.tag_name_str.clone();

        // 追加类选择器
        if let Some(class_attr) = node.get_attr("class") {
            for class in class_attr.split_whitespace() {
                selector.push_str(&format!(".{}", class));
            }
        }

        selector
    }

    /// 构建当前节点的 tag.class 局部路径段（不优先 ID，用于后代选择器路径）
    fn build_local_selector_for_path(&self, node_id: DomNodeId) -> String {
        let node = match self.document.get_node(node_id) {
            Some(n) => n,
            None => return String::new(),
        };
        let mut selector = node.tag_name_str.clone();
        if let Some(class_attr) = node.get_attr("class") {
            for class in class_attr.split_whitespace() {
                selector.push_str(&format!(".{}", class));
            }
        }
        selector
    }

    // ── 公开查询接口 ──────────────────────────────────────────────────────────

    pub fn all_rects(&self) -> Vec<(DomNodeId, String, LayoutRect, Option<(u8, u8, u8, u8)>)> {
        let mut results = Vec::new();
        for (&node_id, box_info) in &self.boxes {
            if let Some(node) = self.document.get_node(node_id) {
                results.push((
                    node_id,
                    node.tag_name_str.clone(),
                    box_info.content_box,
                    box_info.node.background,
                ));
            }
        }
        results
    }

    pub fn hit_test(&self, x: f32, y: f32) -> Option<(DomNodeId, String, LayoutRect)> {
        let mut results = Vec::new();
        for (&node_id, box_info) in &self.boxes {
            if box_info.hit_test(x, y) {
                if let Some(node) = self.document.get_node(node_id) {
                    results.push((node_id, node.tag_name_str.clone(), box_info.content_box));
                }
            }
        }
        results.reverse();
        results.into_iter().next()
    }

    pub fn get_rect(&self, selector: &str) -> Option<LayoutRect> {
        self.document
            .query_selector(selector)
            .and_then(|id: DomNodeId| self.boxes.get(&id).map(|b| b.content_box))
    }

    /// 获取指定节点的布局 box
    pub fn get_box(&self, node_id: DomNodeId) -> Option<&LayoutBox> {
        self.boxes.get(&node_id)
    }
}

/// 解析 CSS 长度字符串，只返回像素数字
fn parse_css_px(value: &str) -> Option<f32> {
    CssLength::parse(value).and_then(|l| match l {
        CssLength::Px(v) => Some(v),
        _ => None,
    })
}
