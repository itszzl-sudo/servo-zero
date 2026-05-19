//! Web → Native 桥接层 - 独立接口定义
//!
//! 定义 [`WebNativeBridge`] trait，供 web-to-native 工具产出的 Rust 代码调用。
//! 不依赖 SpiderMonkey，使用独立的 HTML/CSS 解析和布局引擎。

pub mod bridge;
pub mod servo_impl;
pub mod real_impl;

pub use bridge::*;
pub use servo_impl::ServoBridge;
pub use real_impl::RealServoBridge;
