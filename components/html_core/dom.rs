//! DOM 树结构定义

use std::collections::HashMap;

/// 唯一节点 ID
pub type NodeId = usize;

/// DOM 节点类型
#[derive(Debug, Clone, PartialEq)]
pub enum DomNodeType {
    Document,
    Element,
    Text,
    Comment,
    DocumentType,
    Fragment,
}

/// DOM 节点
#[derive(Debug, Clone)]
pub struct DomNode {
    pub id: NodeId,
    pub node_type: DomNodeType,
    pub tag_name: Option<String>,
    pub tag_name_str: String,
    pub attributes: HashMap<String, String>,
    pub children: Vec<NodeId>,
    pub parent: Option<NodeId>,
    pub text_content: Option<String>,
}

impl DomNode {
    /// 创建文档节点
    pub fn new_document(id: NodeId) -> Self {
        Self {
            id,
            node_type: DomNodeType::Document,
            tag_name: None,
            tag_name_str: String::new(),
            attributes: HashMap::new(),
            children: Vec::new(),
            parent: None,
            text_content: None,
        }
    }

    /// 创建元素节点
    pub fn new_element(id: NodeId, tag_name: String) -> Self {
        Self {
            id,
            node_type: DomNodeType::Element,
            tag_name: Some(tag_name.clone()),
            tag_name_str: tag_name.to_lowercase(),
            attributes: HashMap::new(),
            children: Vec::new(),
            parent: None,
            text_content: None,
        }
    }

    /// 创建文本节点
    pub fn new_text(id: NodeId, text: String) -> Self {
        Self {
            id,
            node_type: DomNodeType::Text,
            tag_name: None,
            tag_name_str: String::new(),
            attributes: HashMap::new(),
            children: Vec::new(),
            parent: None,
            text_content: Some(text),
        }
    }

    /// 创建注释节点
    pub fn new_comment(id: NodeId, text: String) -> Self {
        Self {
            id,
            node_type: DomNodeType::Comment,
            tag_name: None,
            tag_name_str: String::new(),
            attributes: HashMap::new(),
            children: Vec::new(),
            parent: None,
            text_content: Some(text),
        }
    }

    /// 获取属性值
    pub fn get_attr(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }

    /// 设置属性
    pub fn set_attr(&mut self, name: String, value: String) {
        self.attributes.insert(name, value);
    }

    /// 移除属性
    pub fn remove_attr(&mut self, name: &str) -> bool {
        self.attributes.remove(name).is_some()
    }

    /// 检查是否有某属性
    pub fn has_attr(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    /// 是否是 HTML 元素（忽略命名空间）
    pub fn is_element(&self) -> bool {
        self.node_type == DomNodeType::Element
    }

    /// 获取标签名（小写）
    pub fn tag_name(&self) -> &str {
        &self.tag_name_str
    }

    /// 获取内部文本
    pub fn text(&self) -> String {
        self.text_content.clone().unwrap_or_default()
    }

    /// 序列化节点为HTML字符串（用于 inner_html/outer_html）
    fn serialize_html(&self, document: &HtmlDocument) -> String {
        match self.node_type {
            DomNodeType::Text => {
                self.text_content.clone().unwrap_or_default()
            }
            DomNodeType::Comment => {
                format!("<!--{}-->", self.text_content.clone().unwrap_or_default())
            }
            DomNodeType::Element => {
                let mut html = String::new();
                
                // 开始标签
                html.push('<');
                html.push_str(&self.tag_name_str);
                
                // 属性
                for (name, value) in &self.attributes {
                    html.push(' ');
                    html.push_str(name);
                    html.push('=');
                    html.push('"');
                    html.push_str(value);
                    html.push('"');
                }
                
                html.push('>');
                
                // 子节点
                for &child_id in &self.children {
                    if let Some(child) = document.get_node(child_id) {
                        html.push_str(&child.serialize_html(document));
                    }
                }
                
                // 结束标签（自闭合标签除外）
                let self_closing = ["br", "hr", "img", "input", "meta", "link"];
                if !self_closing.contains(&self.tag_name_str.as_str()) {
                    html.push_str("</");
                    html.push_str(&self.tag_name_str);
                    html.push('>');
                }
                
                html
            }
            _ => String::new(),
        }
    }
}

/// HTML 文档
#[derive(Debug, Clone)]
pub struct HtmlDocument {
    nodes: Vec<DomNode>,
    root_id: Option<NodeId>,
    id_counter: NodeId,
}

impl HtmlDocument {
    /// 创建新文档
    pub fn new() -> Self {
        let mut doc = Self {
            nodes: Vec::new(),
            root_id: None,
            id_counter: 0,
        };
        // 创建文档根节点
        let doc_node = DomNode::new_document(doc.next_id());
        doc.root_id = Some(doc.nodes.len());
        doc.nodes.push(doc_node);
        doc
    }

    /// 获取下一个可用 ID
    pub fn next_id(&mut self) -> NodeId {
        let id = self.id_counter;
        self.id_counter += 1;
        id
    }

    /// 添加节点
    pub fn add_node(&mut self, mut node: DomNode) -> NodeId {
        let id = self.next_id();
        node.id = id;
        let idx = self.nodes.len();
        self.nodes.push(node);
        idx
    }

    /// 获取节点
    pub fn get_node(&self, id: NodeId) -> Option<&DomNode> {
        self.nodes.get(id)
    }

    /// 获取可变节点
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut DomNode> {
        self.nodes.get_mut(id)
    }

    /// 根节点 ID
    pub fn root_id(&self) -> Option<NodeId> {
        self.root_id
    }

    /// 所有节点
    pub fn all_nodes(&self) -> &[DomNode] {
        &self.nodes
    }
    
    /// 节点数量
    pub fn nodes_len(&self) -> usize {
        self.nodes.len()
    }

    /// 按 CSS 选择器查找第一个匹配元素（支持复合/后代选择器）
    pub fn query_selector(&self, selector: &str) -> Option<NodeId> {
        let root = self.root_id?;
        let selector = selector.trim();
    
        // 后代选择器（含空格）
        if selector.contains(' ') {
            let parts: Vec<&str> = selector.split_whitespace().collect();
            let last = parts[parts.len() - 1];
            let ancestors = &parts[..parts.len() - 1];
            return self.find_by_descendant(root, ancestors, last);
        }
    
        // 复合选择器 #id.class
        if selector.contains('#') || (selector.starts_with('.') && selector.len() > 1) {
            // 提取 id 和 class
            let mut tag = "*";
            let mut id = None;
            let mut classes: Vec<&str> = Vec::new();
    
            let mut remaining = selector;
            if let Some(idx) = remaining.find('#') {
                tag = &remaining[..idx];
                if tag.is_empty() { tag = "*"; }
                remaining = &remaining[idx + 1..];
                if let Some(next) = remaining.find('.') {
                    id = Some(&remaining[..next]);
                    remaining = &remaining[next..];
                } else {
                    id = Some(remaining);
                    remaining = "";
                }
            }
            for cls in remaining.split('.') {
                if !cls.is_empty() {
                    classes.push(cls);
                }
            }
    
            return self.find_compound(root, tag, id, &classes);
        }
    
        // 简单选择器
        if let Some(id) = selector.strip_prefix('#') {
            self.find_by_id(root, id)
        } else if let Some(class) = selector.strip_prefix('.') {
            self.find_by_class(root, class)
        } else {
            self.find_by_tag(root, selector)
        }
    }

    /// 按 CSS 选择器查找所有匹配元素（支持复合/后代选择器）
    pub fn query_selector_all(&self, selector: &str) -> Vec<NodeId> {
        let root = self.root_id().unwrap_or(0);
        let selector = selector.trim();
    
        // 后代选择器
        if selector.contains(' ') {
            let parts: Vec<&str> = selector.split_whitespace().collect();
            let last = parts[parts.len() - 1];
            let ancestors = &parts[..parts.len() - 1];
            return self.find_all_by_descendant(root, ancestors, last);
        }
    
        // 复合选择器
        if selector.contains('#') || (selector.starts_with('.') && selector.len() > 1) {
            let mut tag = "*";
            let mut id = None;
            let mut classes: Vec<&str> = Vec::new();
    
            let mut remaining = selector;
            if let Some(idx) = remaining.find('#') {
                tag = &remaining[..idx];
                if tag.is_empty() { tag = "*"; }
                remaining = &remaining[idx + 1..];
                if let Some(next) = remaining.find('.') {
                    id = Some(&remaining[..next]);
                    remaining = &remaining[next..];
                } else {
                    id = Some(remaining);
                    remaining = "";
                }
            }
            for cls in remaining.split('.') {
                if !cls.is_empty() {
                    classes.push(cls);
                }
            }
    
            return self.find_all_compound(root, tag, id, &classes);
        }
    
        // 简单选择器
        if let Some(id) = selector.strip_prefix('#') {
            self.find_all_by_id(root, id)
        } else if let Some(class) = selector.strip_prefix('.') {
            self.find_all_by_class(root, class)
        } else {
            self.find_all_by_tag(root, selector)
        }
    }

    fn find_by_id(&self, node_id: NodeId, id: &str) -> Option<NodeId> {
        let node = self.get_node(node_id)?;
        if node.attributes.get("id").map(|s| s.as_str()) == Some(id) {
            return Some(node_id);
        }
        for &child_id in &node.children {
            if let Some(found) = self.find_by_id(child_id, id) {
                return Some(found);
            }
        }
        None
    }

    fn find_by_class(&self, node_id: NodeId, class: &str) -> Option<NodeId> {
        let node = self.get_node(node_id)?;
        if node.attributes.get("class")
            .map(|c| c.split_whitespace().any(|x| x == class))
            .unwrap_or(false) 
        {
            return Some(node_id);
        }
        for &child_id in &node.children {
            if let Some(found) = self.find_by_class(child_id, class) {
                return Some(found);
            }
        }
        None
    }

    fn find_by_tag(&self, node_id: NodeId, tag: &str) -> Option<NodeId> {
        let node = self.get_node(node_id)?;
        if node.tag_name_str == tag {
            return Some(node_id);
        }
        for &child_id in &node.children {
            if let Some(found) = self.find_by_tag(child_id, tag) {
                return Some(found);
            }
        }
        None
    }

    fn find_all_by_id(&self, node_id: NodeId, id: &str) -> Vec<NodeId> {
        let mut results = Vec::new();
        if let Some(node) = self.get_node(node_id) {
            if node.attributes.get("id").map(|s| s.as_str()) == Some(id) {
                results.push(node_id);
            }
            for &child_id in &node.children {
                results.extend(self.find_all_by_id(child_id, id));
            }
        }
        results
    }

    fn find_all_by_class(&self, node_id: NodeId, class: &str) -> Vec<NodeId> {
        let mut results = Vec::new();
        if let Some(node) = self.get_node(node_id) {
            if node.attributes.get("class")
                .map(|c| c.split_whitespace().any(|x| x == class))
                .unwrap_or(false) 
            {
                results.push(node_id);
            }
            for &child_id in &node.children {
                results.extend(self.find_all_by_class(child_id, class));
            }
        }
        results
    }

    fn find_all_by_tag(&self, node_id: NodeId, tag: &str) -> Vec<NodeId> {
        let mut results = Vec::new();
        if let Some(node) = self.get_node(node_id) {
            if node.tag_name_str == tag {
                results.push(node_id);
            }
            for &child_id in &node.children {
                results.extend(self.find_all_by_tag(child_id, tag));
            }
        }
        results
    }

    /// 查找后代选择器匹配的第一个元素
    fn find_by_descendant(&self, node_id: NodeId, ancestor_segs: &[&str], last_seg: &str) -> Option<NodeId> {
        // DFS
        let node = self.get_node(node_id)?;

        // 检查当前节点是否匹配最后一段
        if self.node_matches_segment(node_id, last_seg) {
            // 检查祖先路径
            if self.has_ancestor_path(node_id, ancestor_segs) {
                return Some(node_id);
            }
        }

        // 递归子节点
        for &child_id in &node.children {
            if let Some(found) = self.find_by_descendant(child_id, ancestor_segs, last_seg) {
                return Some(found);
            }
        }
        None
    }

    /// 查找所有后代选择器匹配的元素
    fn find_all_by_descendant(&self, node_id: NodeId, ancestor_segs: &[&str], last_seg: &str) -> Vec<NodeId> {
        let mut results = Vec::new();
        if let Some(node) = self.get_node(node_id) {
            if self.node_matches_segment(node_id, last_seg) {
                if self.has_ancestor_path(node_id, ancestor_segs) {
                    results.push(node_id);
                }
            }
            for &child_id in &node.children {
                results.extend(self.find_all_by_descendant(child_id, ancestor_segs, last_seg));
            }
        }
        results
    }

    /// 检查节点是否有匹配的祖先路径（子序列匹配）
    fn has_ancestor_path(&self, node_id: NodeId, rule_parts: &[&str]) -> bool {
        if rule_parts.is_empty() {
            return true;
        }
        // 构建祖先路径
        let mut ancestors: Vec<NodeId> = Vec::new();
        let mut current = self.get_node(node_id).and_then(|n| n.parent);
        while let Some(parent_id) = current {
            ancestors.push(parent_id);
            current = self.get_node(parent_id).and_then(|n| n.parent);
        }
        ancestors.reverse();

        // 子序列匹配
        let mut rule_idx = 0;
        for &anc_id in &ancestors {
            if rule_idx < rule_parts.len() && self.node_matches_segment(anc_id, rule_parts[rule_idx]) {
                rule_idx += 1;
            }
        }
        rule_idx == rule_parts.len()
    }

    /// 检查节点是否匹配一个简单选择器段
    fn node_matches_segment(&self, node_id: NodeId, rule_seg: &str) -> bool {
        let node = match self.get_node(node_id) {
            Some(n) => n,
            None => return false,
        };
        if rule_seg == "*" {
            return true;
        }
        
        // 属性选择器 [attr=value], [attr], [attr~=value], [attr|=value], [attr^=value], [attr$=value], [attr*=value]
        if rule_seg.starts_with('[') && rule_seg.ends_with(']') {
            return self.matches_attribute_selector(node, &rule_seg[1..rule_seg.len()-1]);
        }
        
        // 伪类选择器 :hover, :first-child, :last-child, :nth-child(), :not(), :empty, :first-of-type, :last-of-type
        if rule_seg.starts_with(':') {
            return self.matches_pseudo_class(node_id, &rule_seg[1..]);
        }
        
        if rule_seg.starts_with('#') {
            return node.attributes.get("id").map(|s| s.as_str()) == Some(&rule_seg[1..]);
        }
        if rule_seg.starts_with('.') {
            let class = &rule_seg[1..];
            return node.attributes.get("class")
                .map(|c| c.split_whitespace().any(|x| x == class))
                .unwrap_or(false);
        }
        // tag.class
        let rule_tag = rule_seg.split('.').next().unwrap_or("");
        let rule_classes: Vec<&str> = rule_seg.split('.').skip(1).collect();
        if !rule_tag.is_empty() && rule_tag != node.tag_name_str {
            return false;
        }
        if let Some(class_attr) = node.attributes.get("class") {
            let elem_classes: Vec<&str> = class_attr.split_whitespace().collect();
            if !rule_classes.iter().all(|rc| elem_classes.contains(rc)) {
                return false;
            }
        } else if !rule_classes.is_empty() {
            return false;
        }
        true
    }
    
    /// 匹配属性选择器
    fn matches_attribute_selector(&self, node: &DomNode, selector: &str) -> bool {
        // 解析属性选择器
        if let Some(eq_pos) = selector.find('=') {
            let attr_name = &selector[..eq_pos].trim();
            let rest = &selector[eq_pos + 1..];
            
            // 提取运算符和值
            let value_str = rest.trim().trim_matches(|c| c == '\'' || c == '"').to_string();
            let (operator, value) = if rest.starts_with('~') {
                ("~=", &value_str)
            } else if rest.starts_with('|') {
                ("|=", &value_str)
            } else if rest.starts_with('^') {
                ("^=", &value_str)
            } else if rest.starts_with('$') {
                ("$=", &value_str)
            } else if rest.starts_with('*') {
                ("*=", &value_str)
            } else {
                ("=", &value_str)
            };
            
            if let Some(attr_value) = node.attributes.get(*attr_name) {
                match operator {
                    "~=" => attr_value.split_whitespace().any(|v| v == value),
                    "|=" => attr_value == value || attr_value.starts_with(&format!("{}-", value)),
                    "^=" => attr_value.starts_with(value),
                    "$=" => attr_value.ends_with(value),
                    "*=" => attr_value.contains(value),
                    "=" => attr_value == value,
                    _ => false,
                }
            } else {
                false
            }
        } else {
            // 仅检查属性是否存在 [attr]
            node.attributes.contains_key(selector.trim())
        }
    }
    
    /// 匹配伪类选择器
    fn matches_pseudo_class(&self, node_id: NodeId, pseudo: &str) -> bool {
        let node = match self.get_node(node_id) {
            Some(n) => n,
            None => return false,
        };
        
        match pseudo {
            // :first-child - 是父节点的第一个子节点
            "first-child" => {
                if let Some(parent_id) = node.parent {
                    if let Some(parent) = self.get_node(parent_id) {
                        if let Some(&first_child_id) = parent.children.first() {
                            return first_child_id == node_id;
                        }
                    }
                }
                false
            }
            // :last-child - 是父节点的最后一个子节点
            "last-child" => {
                if let Some(parent_id) = node.parent {
                    if let Some(parent) = self.get_node(parent_id) {
                        if let Some(&last_child_id) = parent.children.last() {
                            return last_child_id == node_id;
                        }
                    }
                }
                false
            }
            // :only-child - 是父节点的唯一子节点
            "only-child" => {
                if let Some(parent_id) = node.parent {
                    if let Some(parent) = self.get_node(parent_id) {
                        return parent.children.len() == 1;
                    }
                }
                false
            }
            // :empty - 没有子节点
            "empty" => {
                node.children.is_empty()
            }
            // :root - 是文档根节点
            "root" => {
                self.root_id == Some(node_id)
            }
            // :first-of-type - 是父节点中同类型标签的第一个
            "first-of-type" => {
                if let Some(parent_id) = node.parent {
                    if let Some(parent) = self.get_node(parent_id) {
                        for &child_id in &parent.children {
                            if let Some(child) = self.get_node(child_id) {
                                if child.tag_name_str == node.tag_name_str {
                                    return child_id == node_id;
                                }
                            }
                        }
                    }
                }
                false
            }
            // :last-of-type - 是父节点中同类型标签的最后一个
            "last-of-type" => {
                if let Some(parent_id) = node.parent {
                    if let Some(parent) = self.get_node(parent_id) {
                        for &child_id in parent.children.iter().rev() {
                            if let Some(child) = self.get_node(child_id) {
                                if child.tag_name_str == node.tag_name_str {
                                    return child_id == node_id;
                                }
                            }
                        }
                    }
                }
                false
            }
            // :nth-child(n) - 是父节点的第n个子节点
            _ if pseudo.starts_with("nth-child(") && pseudo.ends_with(')') => {
                let n_str = &pseudo[10..pseudo.len()-1].trim();
                if let Ok(n) = n_str.parse::<usize>() {
                    if let Some(parent_id) = node.parent {
                        if let Some(parent) = self.get_node(parent_id) {
                            if n > 0 && n <= parent.children.len() {
                                return parent.children[n - 1] == node_id;
                            }
                        }
                    }
                }
                false
            }
            // :not(selector) - 不匹配指定选择器
            _ if pseudo.starts_with("not(") && pseudo.ends_with(')') => {
                let inner_selector = &pseudo[4..pseudo.len()-1];
                !self.node_matches_segment(node_id, inner_selector)
            }
            // 其他伪类（如 :hover, :focus）需要运行时状态，这里不支持
            _ => false,
        }
    }

    /// 查找复合选择器匹配的第一个元素
    fn find_compound(&self, node_id: NodeId, tag: &str, id: Option<&str>, classes: &[&str]) -> Option<NodeId> {
        if let Some(node) = self.get_node(node_id) {
            // 检查标签
            if tag != "*" && node.tag_name_str != tag {
                return None;
            }
            // 检查 ID
            if let Some(id_val) = id {
                if node.attributes.get("id").map(|s| s.as_str()) != Some(id_val) {
                    return None;
                }
            }
            // 检查类
            if let Some(class_attr) = node.attributes.get("class") {
                let elem_classes: Vec<&str> = class_attr.split_whitespace().collect();
                if !classes.iter().all(|c| elem_classes.contains(c)) {
                    return None;
                }
            } else if !classes.is_empty() {
                return None;
            }
            // 匹配成功
            return Some(node_id);
        }

        // 递归子节点
        if let Some(node) = self.get_node(node_id) {
            for &child_id in &node.children {
                if let Some(found) = self.find_compound(child_id, tag, id, classes) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// 查找复合选择器匹配的所有元素
    fn find_all_compound(&self, node_id: NodeId, tag: &str, id: Option<&str>, classes: &[&str]) -> Vec<NodeId> {
        let mut results = Vec::new();
        if let Some(node) = self.get_node(node_id) {
            let mut matched = true;
            // 检查标签
            if tag != "*" && node.tag_name_str != tag {
                matched = false;
            }
            // 检查 ID
            if matched {
                if let Some(id_val) = id {
                    if node.attributes.get("id").map(|s| s.as_str()) != Some(id_val) {
                        matched = false;
                    }
                }
            }
            // 检查类
            if matched {
                if let Some(class_attr) = node.attributes.get("class") {
                    let elem_classes: Vec<&str> = class_attr.split_whitespace().collect();
                    if !classes.iter().all(|c| elem_classes.contains(c)) {
                        matched = false;
                    }
                } else if !classes.is_empty() {
                    matched = false;
                }
            }
            if matched {
                results.push(node_id);
            }
            for &child_id in &node.children {
                results.extend(self.find_all_compound(child_id, tag, id, classes));
            }
        }
        results
    }

    /// 获取父节点
    pub fn parent(&self, node_id: NodeId) -> Option<NodeId> {
        self.get_node(node_id).and_then(|n| n.parent)
    }

    /// 添加子节点（建立父子关系）
    pub fn append_child(&mut self, parent_id: NodeId, child_id: NodeId) {
        if let Some(parent) = self.nodes.get_mut(parent_id) {
            parent.children.push(child_id);
        }
        if let Some(child) = self.nodes.get_mut(child_id) {
            child.parent = Some(parent_id);
        }
    }

    /// 移除子节点
    pub fn remove_child(&mut self, parent_id: NodeId, child_id: NodeId) -> bool {
        // 从父节点的children中移除
        if let Some(parent) = self.nodes.get_mut(parent_id) {
            if let Some(pos) = parent.children.iter().position(|&id| id == child_id) {
                parent.children.remove(pos);
                // 清除子节点的parent引用
                if let Some(child) = self.nodes.get_mut(child_id) {
                    child.parent = None;
                }
                return true;
            }
        }
        false
    }

    /// 在指定位置插入子节点
    pub fn insert_before(&mut self, parent_id: NodeId, child_id: NodeId, reference_child_id: NodeId) -> bool {
        // 找到reference_child在父节点children中的位置
        if let Some(parent) = self.nodes.get_mut(parent_id) {
            if let Some(pos) = parent.children.iter().position(|&id| id == reference_child_id) {
                parent.children.insert(pos, child_id);
                // 设置子节点的parent引用
                if let Some(child) = self.nodes.get_mut(child_id) {
                    child.parent = Some(parent_id);
                }
                return true;
            }
        }
        false
    }

    /// 替换子节点
    pub fn replace_child(&mut self, parent_id: NodeId, new_child_id: NodeId, old_child_id: NodeId) -> bool {
        // 找到old_child在父节点children中的位置
        if let Some(parent) = self.nodes.get_mut(parent_id) {
            if let Some(pos) = parent.children.iter().position(|&id| id == old_child_id) {
                parent.children[pos] = new_child_id;
                // 设置新子节点的parent引用
                if let Some(new_child) = self.nodes.get_mut(new_child_id) {
                    new_child.parent = Some(parent_id);
                }
                // 清除旧子节点的parent引用
                if let Some(old_child) = self.nodes.get_mut(old_child_id) {
                    old_child.parent = None;
                }
                return true;
            }
        }
        false
    }

    /// 克隆节点（可选：是否克隆后代）
    pub fn clone_node(&mut self, node_id: NodeId, deep: bool) -> Option<NodeId> {
        // 先复制节点数据，避免借用冲突
        let (cloned_node, old_children) = {
            let source_node = self.get_node(node_id)?;
            let cloned_node = source_node.clone();
            let old_children = if deep {
                source_node.children.clone()
            } else {
                Vec::new()
            };
            (cloned_node, old_children)
        };
        
        // 分配新ID
        let mut cloned_node = cloned_node;
        let new_id = self.next_id();
        cloned_node.id = new_id;
        cloned_node.parent = None; // 克隆的节点没有父节点
        cloned_node.children.clear(); // 先清空children
        
        let new_idx = self.nodes.len();
        self.nodes.push(cloned_node);
        
        // 如果deep为true，递归克隆子节点
        if deep {
            for &old_child_id in &old_children {
                if let Some(new_child_id) = self.clone_node(old_child_id, true) {
                    // 建立新的父子关系
                    if let Some(new_parent) = self.nodes.get_mut(new_idx) {
                        new_parent.children.push(new_child_id);
                    }
                    if let Some(new_child) = self.nodes.get_mut(new_child_id) {
                        new_child.parent = Some(new_id);
                    }
                }
            }
        }
        
        Some(new_id)
    }

    /// 检查节点是否包含另一个节点
    pub fn contains(&self, node_id: NodeId, other_id: NodeId) -> bool {
        if node_id == other_id {
            return true;
        }
        
        let mut current = self.get_node(other_id).and_then(|n| n.parent);
        while let Some(parent_id) = current {
            if parent_id == node_id {
                return true;
            }
            current = self.get_node(parent_id).and_then(|n| n.parent);
        }
        false
    }

    /// 检查两个节点是否为同一节点
    pub fn is_same_node(&self, node_id: NodeId, other_id: NodeId) -> bool {
        node_id == other_id
    }

    /// 获取子节点
    pub fn children(&self, node_id: NodeId) -> Vec<NodeId> {
        self.get_node(node_id).map(|n| n.children.clone()).unwrap_or_default()
    }

    /// 获取节点的内部HTML（所有子节点的HTML）
    pub fn inner_html(&self, node_id: NodeId) -> Option<String> {
        let node = self.get_node(node_id)?;
        let mut html = String::new();
        
        for &child_id in &node.children {
            if let Some(child) = self.get_node(child_id) {
                html.push_str(&child.serialize_html(self));
            }
        }
        
        Some(html)
    }

    /// 获取节点的外部HTML（包含节点自身）
    pub fn outer_html(&self, node_id: NodeId) -> Option<String> {
        let node = self.get_node(node_id)?;
        Some(node.serialize_html(self))
    }

    /// 收集节点及其所有后代文本
    pub fn collect_text(&self, node_id: NodeId) -> String {
        let mut result = String::new();
        self.collect_text_impl(node_id, &mut result);
        result
    }

    fn collect_text_impl(&self, node_id: NodeId, result: &mut String) {
        if let Some(node) = self.get_node(node_id) {
            if let Some(ref text) = node.text_content {
                result.push_str(text);
            }
            for &child_id in &node.children {
                self.collect_text_impl(child_id, result);
            }
        }
    }
}
