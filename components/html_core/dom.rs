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

    /// 按 CSS 选择器查找第一个匹配元素
    pub fn query_selector(&self, selector: &str) -> Option<NodeId> {
        let root = self.root_id?;
        
        // 简单的选择器支持
        if let Some(id) = selector.strip_prefix('#') {
            // ID 选择器
            self.find_by_id(root, id)
        } else if let Some(class) = selector.strip_prefix('.') {
            // 类选择器
            self.find_by_class(root, class)
        } else {
            // 标签选择器
            self.find_by_tag(root, selector)
        }
    }

    /// 按 CSS 选择器查找所有匹配元素
    pub fn query_selector_all(&self, selector: &str) -> Vec<NodeId> {
        let root = self.root_id().unwrap_or(0);
        
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

    /// 获取父节点
    pub fn parent(&self, node_id: NodeId) -> Option<NodeId> {
        self.get_node(node_id).and_then(|n| n.parent)
    }

    /// 获取子节点
    pub fn children(&self, node_id: NodeId) -> Vec<NodeId> {
        self.get_node(node_id).map(|n| n.children.clone()).unwrap_or_default()
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
