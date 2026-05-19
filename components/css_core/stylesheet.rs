//! CSS 样式表解析

use std::collections::HashMap;
use crate::declaration::Declaration;

/// CSS 规则
#[derive(Debug, Clone)]
pub struct CssRule {
    pub selector: String,
    pub declarations: Vec<Declaration>,
}

/// 样式表
#[derive(Debug, Clone)]
pub struct Stylesheet {
    pub rules: Vec<CssRule>,
    selector_index: HashMap<String, Vec<usize>>,
}

impl Stylesheet {
    /// 创建新的空样式表
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            selector_index: HashMap::new(),
        }
    }
    
    /// 从 CSS 文本解析样式表
    pub fn from_str(css: &str) -> Self {
        let mut stylesheet = Self::new();
        stylesheet.parse(css);
        stylesheet
    }
    
    /// 解析 CSS 文本
    pub fn parse(&mut self, css: &str) {
        let css = css.trim();
        let mut pos = 0;
        
        while pos < css.len() {
            // 跳过空白和注释
            while pos < css.len() && css[pos..].chars().next().map_or(false, |c| c.is_whitespace()) {
                pos += 1;
            }
            
            if pos >= css.len() {
                break;
            }
            
            // 跳过注释
            if css[pos..].starts_with("/*") {
                if let Some(end) = css[pos..].find("*/") {
                    pos += end + 2;
                    continue;
                }
            }
            
            // 查找选择器结束位置（第一个 {）
            let selector_start = pos;
            let mut found_open = false;
            
            while pos < css.len() {
                let c = css[pos..].chars().next().unwrap();
                if c == '{' {
                    found_open = true;
                    pos += 1;
                    break;
                }
                if c == '}' {
                    break;
                }
                pos += 1;
            }
            
            if !found_open || pos > css.len() {
                break;
            }
            
            let selector = css[selector_start..pos - 1].trim().to_string();
            
            // 解析声明块
            let declaration_start = pos;
            while pos < css.len() {
                let c = css[pos..].chars().next().unwrap();
                if c == '}' {
                    pos += 1;
                    break;
                }
                pos += 1;
            }
            
            let declarations_str = &css[declaration_start..pos - 1];
            let declarations = Self::parse_declarations(declarations_str);
            
            // 添加规则
            let idx = self.rules.len();
            self.rules.push(CssRule {
                selector: selector.clone(),
                declarations: declarations,
            });
            self.selector_index
                .entry(selector)
                .or_default()
                .push(idx);
        }
    }
    
    /// 解析声明块
    fn parse_declarations(declarations_str: &str) -> Vec<Declaration> {
        let mut declarations = Vec::new();
        let mut pos = 0;
        let s = declarations_str;
        
        while pos < s.len() {
            // 跳过空白
            while pos < s.len() && s[pos..].chars().next().map_or(false, |c| c.is_whitespace()) {
                pos += 1;
            }
            
            if pos >= s.len() {
                break;
            }
            
            // 跳过注释
            if s[pos..].starts_with("/*") {
                if let Some(end) = s[pos..].find("*/") {
                    pos += end + 2;
                    continue;
                }
            }
            
            // 查找属性名
            let prop_start = pos;
            while pos < s.len() && s[pos..].chars().next().map_or(false, |c| c != ':') {
                pos += 1;
            }
            
            if pos >= s.len() {
                break;
            }
            
            let prop_name = s[prop_start..pos].trim().to_string();
            pos += 1; // 跳过 :
            
            // 跳过空白
            while pos < s.len() && s[pos..].chars().next().map_or(false, |c| c.is_whitespace()) {
                pos += 1;
            }
            
            // 查找值结束（; 或 }）
            let value_start = pos;
            while pos < s.len() {
                let c = s[pos..].chars().next().unwrap();
                if c == ';' || c == '}' {
                    break;
                }
                pos += 1;
            }
            
            let value = s[value_start..pos].trim().to_string();
            
            if !prop_name.is_empty() && !value.is_empty() {
                declarations.push(Declaration {
                    property: prop_name,
                    value,
                });
            }
            
            if pos < s.len() && s[pos..].chars().next().unwrap() == ';' {
                pos += 1;
            }
        }
        
        declarations
    }
    
    /// 添加 CSS 规则
    pub fn add_rule(&mut self, selector: String, declarations: Vec<Declaration>) {
        let idx = self.rules.len();
        self.rules.push(CssRule {
            selector: selector.clone(),
            declarations,
        });
        self.selector_index
            .entry(selector)
            .or_default()
            .push(idx);
    }
    
    /// 获取匹配的选择器规则
    pub fn get_rules_for(&self, selector: &str) -> Vec<&CssRule> {
        self.selector_index
            .get(selector)
            .map(|indices| indices.iter().filter_map(|&i| self.rules.get(i)).collect())
            .unwrap_or_default()
    }
    
    /// 所有规则
    pub fn all_rules(&self) -> &[CssRule] {
        &self.rules
    }
}

impl Default for Stylesheet {
    fn default() -> Self {
        Self::new()
    }
}
