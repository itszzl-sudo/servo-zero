//! Web → Native 桥接层 - 独立接口定义
//!
//! 定义 [`WebNativeBridge`] trait，供 web-to-native 工具产出的 Rust 代码调用。
//! 不依赖任何浏览器引擎内部类型，外部项目可以实现此 trait 接入自己的渲染引擎。

pub mod bridge;
pub mod network;
pub mod servo_impl;
pub mod real_impl;

pub use bridge::*;
pub use network::*;
pub use servo_impl::ServoBridge;
pub use real_impl::RealServoBridge;
