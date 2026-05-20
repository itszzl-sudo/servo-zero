//! Flexbox 布局类型 — 从 servo/components/layout 思路移植，实际计算委托给 taffy 库
//!
//! 原来的手写 Flexbox 算法已被 taffy 替换，此模块仅保留公开类型供外部使用。

pub use taffy::{FlexDirection, FlexWrap, AlignItems, JustifyContent};

/// Flexbox 布局描述（薄包装）
///
/// 实际布局计算由 [`crate::LayoutTree::layout`] 通过 [`taffy::TaffyTree`] 完成。
#[derive(Debug, Clone)]
pub struct FlexboxLayout {
    pub direction: FlexDirection,
    pub wrap: FlexWrap,
    pub align_items: Option<AlignItems>,
    pub justify_content: Option<JustifyContent>,
}

impl Default for FlexboxLayout {
    fn default() -> Self {
        Self {
            direction: FlexDirection::Row,
            wrap: FlexWrap::NoWrap,
            align_items: None,
            justify_content: None,
        }
    }
}
