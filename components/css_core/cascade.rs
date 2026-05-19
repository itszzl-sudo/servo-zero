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
    
    /// 简单选择器匹配
    fn selector_matches(&self, element_selector: &str, rule_selector: &str) -> bool {
        let element = element_selector.trim();
        let rule = rule_selector.trim();
        
        if element == rule {
            return true;
        }
        
        if rule == "*" {
            return true;
        }
        
        if let Some(id) = rule.strip_prefix('#') {
            return element == format!("#{}", id) || element.ends_with(&format!(" #{}", id)) || element.contains(&format!(" #{} ", id));
        }
        
        if let Some(class) = rule.strip_prefix('.') {
            return element == format!(".{}", class) || element.contains(&format!(" .{} ", class)) || element.ends_with(&format!(" .{}", class));
        }
        
        if rule.contains(',') {
            return rule.split(',').any(|s| self.selector_matches(element, s.trim()));
        }
        
        false
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
            "margin" | "margin-top" => style.margin_top = Some(decl.value.clone()),
            "margin-bottom" => style.margin_bottom = Some(decl.value.clone()),
            "margin-left" => style.margin_left = Some(decl.value.clone()),
            "margin-right" => style.margin_right = Some(decl.value.clone()),
            "padding" | "padding-top" => style.padding_top = Some(decl.value.clone()),
            "padding-bottom" => style.padding_bottom = Some(decl.value.clone()),
            "padding-left" => style.padding_left = Some(decl.value.clone()),
            "padding-right" => style.padding_right = Some(decl.value.clone()),
            "border-width" | "border" => {
                if let Ok(v) = decl.value.parse::<f32>() {
                    style.border_width = Some(v);
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
