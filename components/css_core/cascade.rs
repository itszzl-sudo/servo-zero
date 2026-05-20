//! CSS 级联和样式解析

use std::collections::HashMap;
use crate::declaration::Declaration;
use crate::stylesheet::Stylesheet;
use crate::parse_color;

/// 计算后的样式
#[derive(Debug, Clone)]
pub struct ComputedStyle {
    pub background_color: Option<(u8, u8, u8, u8)>,
    pub color: Option<(u8, u8, u8, u8)>,
    pub width: Option<String>,
    pub height: Option<String>,
    pub display: Option<String>,
    pub position: Option<String>,
    pub left: Option<String>,
    pub right: Option<String>,
    pub top: Option<String>,
    pub bottom: Option<String>,
    pub margin_top: Option<String>,
    pub margin_bottom: Option<String>,
    pub margin_left: Option<String>,
    pub margin_right: Option<String>,
    pub padding_top: Option<String>,
    pub padding_bottom: Option<String>,
    pub padding_left: Option<String>,
    pub padding_right: Option<String>,
    pub border_width: Option<f32>,
    pub border_color: Option<(u8, u8, u8, u8)>,
    pub font_size: Option<String>,
    pub font_weight: Option<String>,
    pub font_family: Option<String>,
    pub text_align: Option<String>,
    pub other: HashMap<String, String>,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self::new()
    }
}

impl ComputedStyle {
    pub fn new() -> Self {
        Self {
            background_color: None,
            color: None,
            width: None,
            height: None,
            display: None,
            position: None,
            left: None,
            right: None,
            top: None,
            bottom: None,
            margin_top: None,
            margin_bottom: None,
            margin_left: None,
            margin_right: None,
            padding_top: None,
            padding_bottom: None,
            padding_left: None,
            padding_right: None,
            border_width: None,
            border_color: None,
            font_size: None,
            font_weight: None,
            font_family: None,
            text_align: None,
            other: HashMap::new(),
        }
    }
    
    /// 获取背景颜色
    pub fn background(&self) -> Option<(u8, u8, u8, u8)> {
        self.background_color
    }
    
    /// 获取文本颜色
    pub fn foreground(&self) -> Option<(u8, u8, u8, u8)> {
        self.color
    }
}

/// 样式解析器
pub struct StyleResolver {
    stylesheets: Vec<Stylesheet>,
    inline_styles: HashMap<String, Vec<Declaration>>,
}

impl StyleResolver {
    pub fn new() -> Self {
        Self {
            stylesheets: Vec::new(),
            inline_styles: HashMap::new(),
        }
    }
    
    /// 添加样式表
    pub fn add_stylesheet(&mut self, stylesheet: Stylesheet) {
        self.stylesheets.push(stylesheet);
    }
    
    /// 添加 CSS 文本
    pub fn add_css(&mut self, css: &str) {
        let stylesheet = Stylesheet::from_str(css);
        self.stylesheets.push(stylesheet);
    }
    
    /// 设置内联样式
    pub fn set_inline_style(&mut self, selector: String, declarations: Vec<Declaration>) {
        self.inline_styles.insert(selector, declarations);
    }
    
    /// 设置元素内联样式（单个属性）
    pub fn set_element_style(&mut self, selector: &str, property: &str, value: &str) {
        let decl = Declaration {
            property: property.to_string(),
            value: value.to_string(),
        };
        self.inline_styles
            .entry(selector.to_string())
            .or_default()
            .push(decl);
    }
    
    /// 计算指定元素的样式
    pub fn compute_style(&self, selector: &str) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        
        // 从样式表收集声明
        let mut all_declarations: Vec<&Declaration> = Vec::new();
        
        for stylesheet in &self.stylesheets {
            for rule in stylesheet.all_rules() {
                if self.selector_matches(selector, &rule.selector) {
                    all_declarations.extend(&rule.declarations);
                }
            }
        }
        
        // 添加内联样式（优先级最高）
        if let Some(inline) = self.inline_styles.get(selector) {
            all_declarations.extend(inline);
        }
        
        // 应用声明
        for decl in all_declarations {
            Self::apply_declaration_to_style(&mut style, decl);
        }
        
        style
    }
    
    /// 简单选择器匹配（已修复假阳性问题）
    fn selector_matches(&self, element_selector: &str, rule_selector: &str) -> bool {
        let element = element_selector.trim();
        let rule = rule_selector.trim();

        // 精确匹配
        if element == rule {
            return true;
        }

        // 通配符
        if rule == "*" {
            return true;
        }

        // 多选择器（逗号分隔，优先检查避免递归歧义）
        if rule.contains(',') {
            return rule.split(',').any(|s| self.selector_matches(element, s.trim()));
        }

        // ID 选择器 #id — build_selector 遇到 id 属性直接返回 "#myid"，精确匹配即可
        if rule.starts_with('#') {
            return element == rule;
        }

        // 类选择器 .class
        if let Some(_class) = rule.strip_prefix('.') {
            // element 为 "div.class1.class2" 或 ".class1"
            // 必须匹配完整类名（避免 .btn 匹配 .btn2）
            return element == rule
                || element.ends_with(rule)                        // "div.class" ends_with ".class"
                || element.contains(&format!("{}.", rule));       // "div.class.other"
        }

        // 标签选择器 div / span / p 等
        // element 可能为 "div" 或 "div.class1.class2"
        // 不能用 starts_with(rule)，避免 "div" 匹配 "divine"
        if !rule.starts_with('.') && !rule.starts_with('#') {
            return element == rule
                || element.starts_with(&format!("{}.", rule)); // "div.class..."
        }

        false
    }

    /// 支持后代选择器的样式计算。
    /// tag_class_selector: 当前元素的 tag.class 形式（如 "div.container"）
    /// id_selector: 当前元素 #id（如 Some("#btn")，无 id 时为 None）
    /// ancestor_path: 祖先元素 tag.class 路径（空格分隔，如 "html body div"）
    /// dom_node_id: DOM 节点 ID，用于查找内联样式（唯一 key）
    pub fn compute_style_with_path(
        &self,
        tag_class_selector: &str,
        id_selector: Option<&str>,
        ancestor_path: &str,
        dom_node_id: Option<usize>,
    ) -> ComputedStyle {
        let mut style = ComputedStyle::new();
        let mut all_declarations: Vec<&Declaration> = Vec::new();

        for stylesheet in &self.stylesheets {
            for rule in stylesheet.all_rules() {
                if self.rule_selector_matches(
                    tag_class_selector,
                    id_selector,
                    ancestor_path,
                    &rule.selector,
                ) {
                    all_declarations.extend(&rule.declarations);
                }
            }
        }

        // 内联样式：优先 #id，其次 dom_node_id key，最后 tag.class
        let inline_key = id_selector
            .map(|s| s.to_string())
            .or_else(|| dom_node_id.map(|id| format!("__nid_{}", id)))
            .unwrap_or_else(|| tag_class_selector.to_string());
        if let Some(inline) = self.inline_styles.get(&inline_key) {
            all_declarations.extend(inline);
        }

        for decl in all_declarations {
            Self::apply_declaration_to_style(&mut style, decl);
        }
        style
    }

    /// 判断规则选择器是否匹配（支持后代选择器、逗号多选择器）
    fn rule_selector_matches(
        &self,
        tag_class: &str,
        id_sel: Option<&str>,
        ancestors: &str,
        rule: &str,
    ) -> bool {
        let rule = rule.trim();

        // 逗号分隔的多选择器
        if rule.contains(',') {
            return rule
                .split(',')
                .any(|s| self.rule_selector_matches(tag_class, id_sel, ancestors, s.trim()));
        }

        // 后代选择器（含空格），如 "div p" / "ul li a"
        if rule.contains(' ') {
            let parts: Vec<&str> = rule.split_whitespace().collect();
            let last = parts[parts.len() - 1];

            // 最后一部分必须匹配当前元素
            if !Self::segment_matches_element(last, tag_class, id_sel) {
                return false;
            }
            if parts.len() == 1 {
                return true;
            }

            // 其余部分在祖先路径中按顺序查找（子序列匹配）
            let anc_segs: Vec<&str> = ancestors.split_whitespace().collect();
            return Self::ancestor_subsequence_matches(&anc_segs, &parts[..parts.len() - 1]);
        }

        // 简单选择器
        Self::segment_matches_element(rule, tag_class, id_sel)
    }

    /// 判断一个 CSS 简单选择器段是否匹配元素
    fn segment_matches_element(rule_seg: &str, tag_class: &str, id_sel: Option<&str>) -> bool {
        if rule_seg == "*" {
            return true;
        }
        // ID 选择器
        if rule_seg.starts_with('#') {
            return id_sel == Some(rule_seg);
        }
        // 纯类选择器（.class）
        if rule_seg.starts_with('.') {
            return tag_class == rule_seg
                || tag_class.ends_with(rule_seg)
                || tag_class.contains(&format!("{}." , rule_seg));
        }
        // 标签（可带类，如 "div.container"）
        let rule_tag = rule_seg.split('.').next().unwrap_or("");
        let rule_classes: Vec<&str> = rule_seg.split('.').skip(1).collect();
        let elem_tag = tag_class.split('.').next().unwrap_or("");
        let elem_classes: Vec<&str> = tag_class.split('.').skip(1).collect();
        if !rule_tag.is_empty() && rule_tag != elem_tag {
            return false;
        }
        rule_classes.iter().all(|rc| elem_classes.contains(rc))
    }

    /// 检查祖先路径是否包含 rule_parts 的子序列（从左到右贪心匹配）
    fn ancestor_subsequence_matches(ancestors: &[&str], rule_parts: &[&str]) -> bool {
        if rule_parts.is_empty() {
            return true;
        }
        if ancestors.is_empty() {
            return false;
        }
        let mut rule_idx = 0;
        for &anc in ancestors {
            if rule_idx < rule_parts.len()
                && Self::seg_matches_tag_class(anc, rule_parts[rule_idx])
            {
                rule_idx += 1;
            }
        }
        rule_idx == rule_parts.len()
    }

    /// 匹配祖先路径段（tag.class）与规则段（不含 #id，后代路径不存 id）
    fn seg_matches_tag_class(tag_class: &str, rule_seg: &str) -> bool {
        if rule_seg == "*" {
            return true;
        }
        if rule_seg.starts_with('#') {
            return false;
        }
        if rule_seg.starts_with('.') {
            return tag_class == rule_seg
                || tag_class.ends_with(rule_seg)
                || tag_class.contains(&format!("{}." , rule_seg));
        }
        let rule_tag = rule_seg.split('.').next().unwrap_or("");
        let rule_classes: Vec<&str> = rule_seg.split('.').skip(1).collect();
        let elem_tag = tag_class.split('.').next().unwrap_or("");
        let elem_classes: Vec<&str> = tag_class.split('.').skip(1).collect();
        if !rule_tag.is_empty() && rule_tag != elem_tag {
            return false;
        }
        rule_classes.iter().all(|rc| elem_classes.contains(rc))
    }

    /// 应用单条声明到计算样式
    fn apply_declaration_to_style(style: &mut ComputedStyle, decl: &Declaration) {
        match decl.property.as_str() {
            "background" | "background-color" => {
                style.background_color = parse_color(&decl.value);
            }
            "color" => {
                style.color = parse_color(&decl.value);
            }
            "width" => {
                style.width = Some(decl.value.clone());
            }
            "height" => {
                style.height = Some(decl.value.clone());
            }
            "display" => {
                style.display = Some(decl.value.clone());
            }
            "position" => {
                style.position = Some(decl.value.clone());
            }
            "left" => style.left = Some(decl.value.clone()),
            "right" => style.right = Some(decl.value.clone()),
            "top" => style.top = Some(decl.value.clone()),
            "bottom" => style.bottom = Some(decl.value.clone()),
            "margin" => {
                // CSS margin shorthand: 1-4 values (top right bottom left)
                let sides = expand_shorthand_4(decl.value.as_str());
                style.margin_top    = Some(sides[0].clone());
                style.margin_right  = Some(sides[1].clone());
                style.margin_bottom = Some(sides[2].clone());
                style.margin_left   = Some(sides[3].clone());
            }
            "margin-top"    => style.margin_top    = Some(decl.value.clone()),
            "margin-bottom" => style.margin_bottom = Some(decl.value.clone()),
            "margin-left"   => style.margin_left   = Some(decl.value.clone()),
            "margin-right"  => style.margin_right  = Some(decl.value.clone()),
            "padding" => {
                // CSS padding shorthand: 1-4 values (top right bottom left)
                let sides = expand_shorthand_4(decl.value.as_str());
                style.padding_top    = Some(sides[0].clone());
                style.padding_right  = Some(sides[1].clone());
                style.padding_bottom = Some(sides[2].clone());
                style.padding_left   = Some(sides[3].clone());
            }
            "padding-top"    => style.padding_top    = Some(decl.value.clone()),
            "padding-bottom" => style.padding_bottom = Some(decl.value.clone()),
            "padding-left"   => style.padding_left   = Some(decl.value.clone()),
            "padding-right"  => style.padding_right  = Some(decl.value.clone()),
            "border-width" => {
                let value = decl.value.trim();
                // CSS border-width 关键字：thin / medium / thick
                match value {
                    "thin" => style.border_width = Some(1.0),
                    "medium" => style.border_width = Some(3.0),
                    "thick" => style.border_width = Some(5.0),
                    _ => {
                        if let Ok(v) = value.trim_end_matches("px").parse::<f32>() {
                            style.border_width = Some(v);
                        }
                    }
                }
            }
            "border" => {
                // border: <width> <style> <color>  — 提取宽度、样式和颜色
                let value = decl.value.trim();
                // 特殊值 none / hidden
                if value == "none" || value == "hidden" {
                    style.border_width = Some(0.0);
                    return;
                }
                for part in value.split_whitespace() {
                    let w = part.trim_end_matches("px");
                    if let Ok(v) = w.parse::<f32>() {
                        style.border_width = Some(v);
                    } else if let Some(c) = parse_color(part) {
                        style.border_color = Some(c);
                    } else if matches!(part, "solid" | "dashed" | "dotted") {
                        // border-style 已记录，渲染时可用（暂存到 other）
                        style.other.insert("border-style".to_string(), part.to_string());
                    } else if part == "none" || part == "hidden" {
                        style.border_width = Some(0.0);
                    }
                }
            }
            "border-color" => {
                style.border_color = parse_color(&decl.value);
            }
            "font-size" => {
                style.font_size = Some(decl.value.clone());
            }
            "font-weight" => {
                style.font_weight = Some(decl.value.clone());
            }
            "font-family" => {
                style.font_family = Some(decl.value.clone());
            }
            "text-align" => {
                style.text_align = Some(decl.value.clone());
            }
            // 新增属性支持
            "opacity" | "z-index" | "visibility" | "line-height" | "letter-spacing" | "text-decoration" | "cursor" | "box-sizing" | "white-space" | "border-radius" | "border-top-left-radius" | "border-top-right-radius" | "border-bottom-left-radius" | "border-bottom-right-radius" => {
                style.other.insert(decl.property.clone(), decl.value.clone());
            }
            _ => {
                style.other.insert(decl.property.clone(), decl.value.clone());
            }
        }
    }
    
    /// 清除所有样式
    pub fn clear(&mut self) {
        self.stylesheets.clear();
        self.inline_styles.clear();
    }
}

impl Default for StyleResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// CSS 四边简写展开（margin / padding）
/// 规则：1个值→全部，2个值→上下/左右，3个值→上/左右/下，4个值→上右下左
fn expand_shorthand_4(value: &str) -> [String; 4] {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 => [parts[0].into(), parts[0].into(), parts[0].into(), parts[0].into()],
        2 => [parts[0].into(), parts[1].into(), parts[0].into(), parts[1].into()],
        3 => [parts[0].into(), parts[1].into(), parts[2].into(), parts[1].into()],
        4 => [parts[0].into(), parts[1].into(), parts[2].into(), parts[3].into()],
        _ => [String::new(), String::new(), String::new(), String::new()],
    }
}
