//! CSS 样式表解析 —— 使用 cssparser（与 Servo 相同的库）

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
    
    /// 使用方法解析 CSS 文本
    pub fn parse(&mut self, css: &str) {
        self.parse_simple(css);
    }
    
    /// 手写 CSS 解析（功能完善版，支持注释、多选择器等）
    fn parse_simple(&mut self, css: &str) {
        // 删除注释
        let css = remove_css_comments(css);
        let css = css.trim();
        let mut pos = 0;
        let bytes = css.as_bytes();
        
        while pos < css.len() {
            // 跳过空白
            while pos < css.len() && (bytes[pos] as char).is_whitespace() {
                pos += 1;
            }
            
            if pos >= css.len() { break; }
            
            // 跳过 @ 规则：处理 @media，跳过其他
            if bytes[pos] == b'@' {
                // 检查是否为 @media
                let at_start = pos;
                let at_end = css.len().min(pos + 10);
                if css[at_start..at_end].to_lowercase().starts_with("@media") {
                    // 解析 @media 规则：提取内部所有样式规则
                    pos = parse_media_rule(css, pos, &mut self.rules, &mut self.selector_index);
                    continue;
                } else {
                    // 其他 @ 规则（@keyframes, @font-face 等）直接跳过
                    pos = skip_at_rule(css, pos);
                    continue;
                }
            }
            
            // 读取选择器（直到 {)
            let sel_start = pos;
            while pos < css.len() && bytes[pos] != b'{' {
                pos += 1;
            }
            if pos >= css.len() { break; }
            
            let selector_block = &css[sel_start..pos];
            pos += 1; // 跳过 {
            
            // 读取声明块（直到 })
            let decl_start = pos;
            while pos < css.len() && bytes[pos] != b'}' {
                pos += 1;
            }
            
            let declarations_str = if pos <= css.len() { &css[decl_start..pos] } else { "" };
            if pos < css.len() { pos += 1; } // 跳过 }
            
            let mut declarations = parse_declarations(declarations_str);
            
            // 检查是否有 !important 标记（解析后剥离标记）
            for decl in &mut declarations {
                if decl.value.ends_with("!important") {
                    decl.value = decl.value[..decl.value.len() - 10].trim().to_string();
                }
            }
            
            // 支持多选择器（用逗号分隔）
            for selector in selector_block.split(',') {
                let selector = selector.trim().to_string();
                if selector.is_empty() { continue; }
                
                let idx = self.rules.len();
                self.rules.push(CssRule {
                    selector: selector.clone(),
                    declarations: declarations.clone(),
                });
                self.selector_index.entry(selector).or_default().push(idx);
            }
        }
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
    
    /// 插入 CSS 规则到指定位置
    pub fn insert_rule(&mut self, index: usize, selector: String, declarations: Vec<Declaration>) -> bool {
        if index > self.rules.len() {
            return false;
        }
        
        self.rules.insert(index, CssRule {
            selector: selector.clone(),
            declarations,
        });
        
        // 重建索引
        self.rebuild_index();
        
        true
    }
    
    /// 删除指定位置的 CSS 规则
    pub fn delete_rule(&mut self, index: usize) -> bool {
        if index >= self.rules.len() {
            return false;
        }
        
        self.rules.remove(index);
        
        // 重建索引
        self.rebuild_index();
        
        true
    }
    
    /// 获取指定位置的 CSS 规则
    pub fn get_rule_at(&self, index: usize) -> Option<&CssRule> {
        self.rules.get(index)
    }
    
    /// 重建选择器索引
    fn rebuild_index(&mut self) {
        self.selector_index.clear();
        for (index, rule) in self.rules.iter().enumerate() {
            self.selector_index
                .entry(rule.selector.clone())
                .or_default()
                .push(index);
        }
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

/// 删除 CSS 注释
fn remove_css_comments(css: &str) -> String {
    let mut result = String::with_capacity(css.len());
    let mut chars = css.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'*') {
            chars.next(); // 消费 *
            // 扮找 */
            while let Some(c2) = chars.next() {
                if c2 == '*' && chars.peek() == Some(&'/') {
                    chars.next(); // 消费 /
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// 跳过一个 @规则块
fn skip_at_rule(css: &str, mut pos: usize) -> usize {
    let bytes = css.as_bytes();
    let mut depth = 0;
    while pos < css.len() {
        match bytes[pos] {
            b'{' => { depth += 1; pos += 1; }
            b'}' => {
                pos += 1;
                if depth > 0 {
                    depth -= 1;
                    if depth == 0 { break; }
                } else {
                    break;
                }
            }
            b';' if depth == 0 => { pos += 1; break; }
            _ => { pos += 1; }
        }
    }
    pos
}

/// 解析声明块内容
pub fn parse_declarations(declarations_str: &str) -> Vec<Declaration> {
    let mut declarations = Vec::new();
    
    for decl in declarations_str.split(';') {
        let decl = decl.trim();
        if decl.is_empty() { continue; }
        
        if let Some(colon) = decl.find(':') {
            let prop = decl[..colon].trim();
            let value = decl[colon + 1..].trim();
            
            if !prop.is_empty() && !value.is_empty() {
                declarations.push(Declaration {
                    property: prop.to_string(),
                    value: value.to_string(),
                });
            }
        }
    }
    
    declarations
}

/// 解析 @media 规则，提取内部的样式规则
/// 对于轻量级渲染引擎，我们无条件应用所有 @media 规则（桌面优先策略）
fn parse_media_rule(
    css: &str,
    start: usize,
    rules: &mut Vec<CssRule>,
    selector_index: &mut std::collections::HashMap<String, Vec<usize>>,
) -> usize {
    let bytes = css.as_bytes();
    let mut pos = start;

    // 跳过 @media 关键字
    while pos < css.len() && bytes[pos] != b'{' {
        pos += 1;
    }
    if pos >= css.len() {
        return pos;
    }
    pos += 1; // 跳过 {

    // 解析内部的所有样式规则
    parse_inner_rules(css, pos, rules, selector_index)
}

/// 解析嵌套在 @media 或其他块内的样式规则
fn parse_inner_rules(
    css: &str,
    mut pos: usize,
    rules: &mut Vec<CssRule>,
    selector_index: &mut std::collections::HashMap<String, Vec<usize>>,
) -> usize {
    let bytes = css.as_bytes();
    let mut depth = 1;

    while pos < css.len() && depth > 0 {
        // 跳过空白
        while pos < css.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }

        if pos >= css.len() {
            break;
        }

        // 遇到闭合括号
        if bytes[pos] == b'}' {
            depth -= 1;
            pos += 1;
            continue;
        }

        // 读取选择器（直到 {)
        let sel_start = pos;
        while pos < css.len() && bytes[pos] != b'{' {
            pos += 1;
        }
        if pos >= css.len() {
            break;
        }

        let selector_block = &css[sel_start..pos];
        pos += 1; // 跳过 {

        // 读取声明块
        let decl_start = pos;
        let mut brace_depth = 1;
        while pos < css.len() && brace_depth > 0 {
            match bytes[pos] {
                b'{' => brace_depth += 1,
                b'}' => brace_depth -= 1,
                _ => {}
            }
            if brace_depth > 0 {
                pos += 1;
            }
        }

        let declarations_str = &css[decl_start..pos];
        if pos < css.len() {
            pos += 1;
        } // 跳过 }

        let mut declarations = parse_declarations(declarations_str);

        // 剥离 !important 标记
        for decl in &mut declarations {
            if decl.value.ends_with("!important") {
                decl.value = decl.value[..decl.value.len() - 10].trim().to_string();
            }
        }

        // 支持多选择器
        for selector in selector_block.split(',') {
            let selector = selector.trim().to_string();
            if selector.is_empty() {
                continue;
            }

            let idx = rules.len();
            rules.push(CssRule {
                selector: selector.clone(),
                declarations: declarations.clone(),
            });
            selector_index.entry(selector).or_default().push(idx);
        }
    }

    pos
}
