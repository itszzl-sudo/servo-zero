//! CSS 核心模块
//! 
//! 提供独立的 CSS 解析、级联和样式计算，不依赖 SpiderMonkey

pub mod stylesheet;
pub mod declaration;
pub mod cascade;

pub use stylesheet::Stylesheet;
pub use declaration::Declaration;
pub use cascade::{StyleResolver, ComputedStyle};

/// 常用颜色名称到 RGB 的映射
pub fn color_from_name(name: &str) -> Option<(u8, u8, u8)> {
    match name.to_lowercase().as_str() {
        "black" => Some((0, 0, 0)),
        "white" => Some((255, 255, 255)),
        "red" => Some((255, 0, 0)),
        "green" => Some((0, 128, 0)),
        "blue" => Some((0, 0, 255)),
        "yellow" => Some((255, 255, 0)),
        "cyan" => Some((0, 255, 255)),
        "magenta" => Some((255, 0, 255)),
        "gray" | "grey" => Some((128, 128, 128)),
        "orange" => Some((255, 165, 0)),
        "purple" => Some((128, 0, 128)),
        "pink" => Some((255, 192, 203)),
        "brown" => Some((165, 42, 42)),
        "navy" => Some((0, 0, 128)),
        "teal" => Some((0, 128, 128)),
        "olive" => Some((128, 128, 0)),
        "maroon" => Some((128, 0, 0)),
        "silver" => Some((192, 192, 192)),
        "lime" => Some((0, 255, 0)),
        "aqua" => Some((0, 255, 255)),
        "fuchsia" => Some((255, 0, 255)),
        "transparent" => Some((0, 0, 0)),
        "currentcolor" => Some((0, 0, 0)), // 特殊处理
        _ => None,
    }
}

/// 解析 CSS 颜色值
pub fn parse_color(value: &str) -> Option<(u8, u8, u8, u8)> {
    let value = value.trim().to_lowercase();
    
    // 尝试解析 #RRGGBB 或 #RGB 格式
    if value.starts_with('#') {
        let hex = value.trim_start_matches('#');
        return parse_hex_color(hex);
    }
    
    // 尝试解析 rgb(), rgba() 格式
    if value.starts_with("rgb(") || value.starts_with("rgba(") {
        let inner = value
            .trim_start_matches("rgb(")
            .trim_start_matches("rgba(")
            .trim_end_matches(')');
        
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() >= 3 {
            let r = parse_color_component(parts[0]).ok()?;
            let g = parse_color_component(parts[1]).ok()?;
            let b = parse_color_component(parts[2]).ok()?;
            let a = if parts.len() >= 4 {
                parse_alpha_component(parts[3]).unwrap_or(1.0)
            } else {
                1.0
            };
            return Some((r, g, b, (a * 255.0) as u8));
        }
    }
    
    // 尝试解析颜色名称
    if let Some((r, g, b)) = color_from_name(&value) {
        return Some((r, g, b, 255));
    }
    
    None
}

fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8, u8)> {
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some((r, g, b, 255))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b, 255))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some((r, g, b, a))
        }
        _ => None,
    }
}

fn parse_color_component(s: &str) -> Result<u8, ()> {
    let s = s.trim();
    
    // 尝试百分比
    if s.ends_with('%') {
        let pct: f32 = s.trim_end_matches('%').parse().map_err(|_| ())?;
        return Ok((pct / 100.0 * 255.0) as u8);
    }
    
    // 尝试直接数值
    let val: f32 = s.parse().map_err(|_| ())?;
    if val > 1.0 {
        Ok(val as u8)
    } else {
        Ok((val * 255.0) as u8)
    }
}

fn parse_alpha_component(s: &str) -> Option<f32> {
    let s = s.trim();
    
    if s.ends_with('%') {
        let pct: f32 = s.trim_end_matches('%').parse().ok()?;
        Some(pct / 100.0)
    } else {
        let val: f32 = s.parse().ok()?;
        Some(val)
    }
}

/// CSS 长度单位解析
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    Px(f32),
    Em(f32),
    Rem(f32),
    Percent(f32),
    Auto,
}

impl Length {
    pub fn parse(value: &str) -> Option<Self> {
        let value = value.trim();
        
        if value == "auto" {
            return Some(Length::Auto);
        }
        
        if value.ends_with("px") {
            let num: f32 = value.trim_end_matches("px").parse().ok()?;
            Some(Length::Px(num))
        } else if value.ends_with("em") {
            let num: f32 = value.trim_end_matches("em").parse().ok()?;
            Some(Length::Em(num))
        } else if value.ends_with("rem") {
            let num: f32 = value.trim_end_matches("rem").parse().ok()?;
            Some(Length::Rem(num))
        } else if value.ends_with('%') {
            let num: f32 = value.trim_end_matches('%').parse().ok()?;
            Some(Length::Percent(num))
        } else {
            // 假设是像素
            let num: f32 = value.parse().ok()?;
            Some(Length::Px(num))
        }
    }
    
    /// 转换为像素值（需要父元素尺寸上下文）
    pub fn to_px(&self, parent_value: f32, font_size: f32) -> f32 {
        match self {
            Length::Px(v) => *v,
            Length::Em(v) => *v * font_size,
            Length::Rem(v) => *v * font_size,
            Length::Percent(v) => *v / 100.0 * parent_value,
            Length::Auto => 0.0,
        }
    }
}
