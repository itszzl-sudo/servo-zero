//! HTML 解析器 —— 基于 html5ever（与 Servo 相同的库）

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};

use crate::dom::{DomNode, HtmlDocument, NodeId};

/// HTML 解析器
pub struct HtmlParser {
    document: HtmlDocument,
}

impl HtmlParser {
    pub fn new() -> Self {
        Self {
            document: HtmlDocument::new(),
        }
    }

    /// 解析 HTML 字符串，返回构建好的文档引用
    pub fn parse(&mut self, html: &str) -> &HtmlDocument {
        // 用 html5ever 解析到 RcDom
        let rc_dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .unwrap_or_else(|_| RcDom::default());

        // 重新初始化文档
        self.document = HtmlDocument::new();

        // 从 RcDom 转换到我们的 HtmlDocument
        let rc_root = rc_dom.document.clone();
        let our_root_id = self.document.root_id().unwrap_or(0);
        self.convert_node(&rc_root, our_root_id, &rc_dom);

        log::info!(
            "html5ever 解析完成，共 {} 个节点",
            self.document.nodes_len()
        );

        &self.document
    }

    pub fn document(&self) -> &HtmlDocument {
        &self.document
    }

    pub fn document_mut(&mut self) -> &mut HtmlDocument {
        &mut self.document
    }

    /// 递归将 RcDom 节点转换为我们的 HtmlDocument 节点
    fn convert_node(&mut self, rc_node: &Handle, parent_id: NodeId, dom: &RcDom) {
        match &rc_node.data {
            NodeData::Document => {
                // 文档根节点已经在 HtmlDocument::new() 里创建了
                for child in rc_node.children.borrow().iter() {
                    self.convert_node(child, parent_id, dom);
                }
            }

            NodeData::Element { name, attrs, .. } => {
                let tag = name.local.as_ref().to_lowercase();

                // 跳过 head / script / style 的内容（不渲染）
                // 但保留 style 标签本身（用于提取 CSS）
                let mut node = DomNode::new_element(0, tag.clone());

                // 拷贝属性
                for attr in attrs.borrow().iter() {
                    let attr_name = attr.name.local.as_ref().to_lowercase();
                    let attr_value = attr.value.as_ref().to_string();
                    node.set_attr(attr_name, attr_value);
                }

                let node_id = self.document.add_node(node);

                // 建立父子关系
                self.document.append_child(parent_id, node_id);

                // 递归子节点
                for child in rc_node.children.borrow().iter() {
                    self.convert_node(child, node_id, dom);
                }
            }

            NodeData::Text { contents } => {
                let text = contents.borrow().as_ref().to_string();
                if !text.trim().is_empty() {
                    let node = DomNode::new_text(0, text);
                    let node_id = self.document.add_node(node);
                    self.document.append_child(parent_id, node_id);
                }
            }

            NodeData::Comment { .. } => {
                // 忽略注释
            }

            NodeData::Doctype { .. } => {
                // 忽略 DOCTYPE
            }

            NodeData::ProcessingInstruction { .. } => {
                // 忽略处理指令
            }
        }
    }
}

impl Default for HtmlParser {
    fn default() -> Self {
        Self::new()
    }
}

/// 从 HTML 字符串中提取所有 <style> 标签的内容
pub fn extract_style_tags(html: &str) -> Vec<String> {
    let mut styles = Vec::new();
    let rc_dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut html.as_bytes())
        .unwrap_or_else(|_| RcDom::default());

    collect_style_contents(&rc_dom.document, &mut styles);
    styles
}

fn collect_style_contents(node: &Handle, styles: &mut Vec<String>) {
    match &node.data {
        NodeData::Element { name, .. } => {
            let tag = name.local.as_ref().to_lowercase();
            if tag == "style" {
                // 收集 style 标签内的文本内容
                let mut css = String::new();
                for child in node.children.borrow().iter() {
                    if let NodeData::Text { contents } = &child.data {
                        css.push_str(contents.borrow().as_ref());
                    }
                }
                if !css.trim().is_empty() {
                    styles.push(css);
                }
                return; // style 标签的子节点不需要继续递归
            }
        }
        _ => {}
    }
    // 递归子节点
    for child in node.children.borrow().iter() {
        collect_style_contents(child, styles);
    }
}
