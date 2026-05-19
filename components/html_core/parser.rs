//! HTML 解析器实现

use crate::dom::{HtmlDocument, DomNode, NodeId};

/// HTML 解析器
pub struct HtmlParser {
    document: HtmlDocument,
}

impl HtmlParser {
    /// 创建新的解析器
    pub fn new() -> Self {
        Self {
            document: HtmlDocument::new(),
        }
    }

    /// 解析 HTML 字符串
    pub fn parse(&mut self, html: &str) -> &HtmlDocument {
        self.parse_html(html, None);
        &self.document
    }

    /// 递归解析 HTML
    fn parse_html(&mut self, html: &str, parent_id: Option<NodeId>) {
        let mut remaining = html.trim();
        let root_id = parent_id.unwrap_or(self.document.root_id().unwrap_or(0));
        
        while !remaining.is_empty() {
            remaining = remaining.trim_start();
            if remaining.is_empty() {
                break;
            }
            
            // 跳过注释
            if remaining.starts_with("<!--") {
                if let Some(end) = remaining.find("-->") {
                    let comment = &remaining[4..end];
                    let mut node = DomNode::new_comment(self.document.next_id(), comment.to_string());
                    node.parent = parent_id;
                    let idx = self.document.add_node(node);
                    if let Some(p) = self.document.get_node_mut(root_id) {
                        p.children.push(idx);
                    }
                    remaining = &remaining[end + 3..];
                    continue;
                }
            }
            
            // 处理文本节点
            if !remaining.starts_with('<') {
                if let Some(end) = remaining.find('<') {
                    let text = &remaining[..end];
                    if !text.trim().is_empty() {
                        let mut node = DomNode::new_text(self.document.next_id(), text.to_string());
                        node.parent = parent_id;
                        let idx = self.document.add_node(node);
                        if let Some(p) = self.document.get_node_mut(root_id) {
                            p.children.push(idx);
                        }
                    }
                    remaining = &remaining[end..];
                    continue;
                }
            }
            
            // 处理标签
            if remaining.starts_with('<') {
                if let Some(end_tag) = remaining.find('>') {
                    let tag_content = &remaining[1..end_tag];
                    
                    // 跳过 DOCTYPE 和其他特殊标签
                    if tag_content.starts_with('!') || tag_content.starts_with("?") {
                        remaining = &remaining[end_tag + 1..];
                        continue;
                    }
                    
                    // 检查是否是闭合标签
                    if tag_content.starts_with('/') {
                        return;
                    }
                    
                    // 解析标签名和属性
                    let parts: Vec<&str> = tag_content.split_whitespace().collect();
                    if !parts.is_empty() {
                        let tag_name = parts[0].to_lowercase();
                        let mut attrs = std::collections::HashMap::new();
                        
                        for attr_part in parts.iter().skip(1) {
                            let attr = attr_part.trim_end_matches('/');
                            if let Some((k, v)) = attr.split_once('=') {
                                let value = v.trim_matches('"').trim_matches('\'');
                                attrs.insert(k.to_string(), value.to_string());
                            }
                        }
                        
                        // 创建节点
                        let mut node = DomNode::new_element(self.document.next_id(), tag_name.clone());
                        node.attributes = attrs;
                        node.parent = parent_id;
                        
                        let idx = self.document.add_node(node);
                        
                        // 自闭合标签
                        let self_closing = tag_content.ends_with('/') 
                            || ["img", "br", "hr", "input", "meta", "link", "area", "base", "col", "embed", "param", "source", "track", "wbr"].contains(&tag_name.as_str());
                        
                        if !self_closing {
                            remaining = &remaining[end_tag + 1..];
                            
                            let close_tag = format!("</{}>", tag_name);
                            
                            if let Some(close_pos) = remaining.find(&close_tag) {
                                let child_content = &remaining[..close_pos];
                                self.parse_html(child_content, Some(idx));
                                remaining = &remaining[close_pos + close_tag.len()..];
                            }
                        } else {
                            remaining = &remaining[end_tag + 1..];
                        }
                        
                        if let Some(p) = self.document.get_node_mut(root_id) {
                            p.children.push(idx);
                        }
                    } else {
                        remaining = &remaining[end_tag + 1..];
                    }
                } else {
                    break;
                }
            } else {
                remaining = &remaining[1..];
            }
        }
    }

    /// 获取解析后的文档
    pub fn document(&self) -> &HtmlDocument {
        &self.document
    }

    /// 获取可变文档引用
    pub fn document_mut(&mut self) -> &mut HtmlDocument {
        &mut self.document
    }
}

impl Default for HtmlParser {
    fn default() -> Self {
        Self::new()
    }
}
