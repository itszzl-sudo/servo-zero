//! CSS 核心模块
//! 
//! 提供独立的 CSS 解析、级联和样式计算，不依赖 SpiderMonkey

pub mod stylesheet;
pub mod declaration;
pub mod cascade;
pub mod transform;
pub mod transition;
pub mod animation;

pub use stylesheet::Stylesheet;
pub use declaration::Declaration;
pub use cascade::{StyleResolver, ComputedStyle};
pub use transform::{TransformFunction, transforms_to_matrix};
pub use transition::Transition;
pub use animation::{Animation, KeyframesAnimation, parse_keyframes};

/// CSS 颜色到 RGB 的映射（从 cssparser-0.31.2/src/color.rs 拷贝，共 148 个颜色）
/// 注意：transparent 和 currentcolor 不在此表中，由 parse_color() 单独处理
pub fn color_from_name(name: &str) -> Option<(u8, u8, u8)> {
    match name.to_lowercase().as_str() {
        // CSS Level 1
        "black"   => Some((0, 0, 0)),
        "silver"  => Some((192, 192, 192)),
        "gray"    => Some((128, 128, 128)),
        "white"   => Some((255, 255, 255)),
        "maroon"  => Some((128, 0, 0)),
        "red"     => Some((255, 0, 0)),
        "purple"  => Some((128, 0, 128)),
        "fuchsia" => Some((255, 0, 255)),
        "green"   => Some((0, 128, 0)),
        "lime"    => Some((0, 255, 0)),
        "olive"   => Some((128, 128, 0)),
        "yellow"  => Some((255, 255, 0)),
        "navy"    => Some((0, 0, 128)),
        "blue"    => Some((0, 0, 255)),
        "teal"    => Some((0, 128, 128)),
        "aqua"    => Some((0, 255, 255)),
        // CSS Level 3 / X11 extended colors
        "aliceblue"            => Some((240, 248, 255)),
        "antiquewhite"         => Some((250, 235, 215)),
        "aquamarine"           => Some((127, 255, 212)),
        "azure"                => Some((240, 255, 255)),
        "beige"                => Some((245, 245, 220)),
        "bisque"               => Some((255, 228, 196)),
        "blanchedalmond"       => Some((255, 235, 205)),
        "blueviolet"           => Some((138, 43, 226)),
        "brown"                => Some((165, 42, 42)),
        "burlywood"            => Some((222, 184, 135)),
        "cadetblue"            => Some((95, 158, 160)),
        "chartreuse"           => Some((127, 255, 0)),
        "chocolate"            => Some((210, 105, 30)),
        "coral"                => Some((255, 127, 80)),
        "cornflowerblue"       => Some((100, 149, 237)),
        "cornsilk"             => Some((255, 248, 220)),
        "crimson"              => Some((220, 20, 60)),
        "cyan"                 => Some((0, 255, 255)),
        "darkblue"             => Some((0, 0, 139)),
        "darkcyan"             => Some((0, 139, 139)),
        "darkgoldenrod"        => Some((184, 134, 11)),
        "darkgray"             => Some((169, 169, 169)),
        "darkgreen"            => Some((0, 100, 0)),
        "darkgrey"             => Some((169, 169, 169)),
        "darkkhaki"            => Some((189, 183, 107)),
        "darkmagenta"          => Some((139, 0, 139)),
        "darkolivegreen"       => Some((85, 107, 47)),
        "darkorange"           => Some((255, 140, 0)),
        "darkorchid"           => Some((153, 50, 204)),
        "darkred"              => Some((139, 0, 0)),
        "darksalmon"           => Some((233, 150, 122)),
        "darkseagreen"         => Some((143, 188, 143)),
        "darkslateblue"        => Some((72, 61, 139)),
        "darkslategray"        => Some((47, 79, 79)),
        "darkslategrey"        => Some((47, 79, 79)),
        "darkturquoise"        => Some((0, 206, 209)),
        "darkviolet"           => Some((148, 0, 211)),
        "deeppink"             => Some((255, 20, 147)),
        "deepskyblue"          => Some((0, 191, 255)),
        "dimgray"              => Some((105, 105, 105)),
        "dimgrey"              => Some((105, 105, 105)),
        "dodgerblue"           => Some((30, 144, 255)),
        "firebrick"            => Some((178, 34, 34)),
        "floralwhite"          => Some((255, 250, 240)),
        "forestgreen"          => Some((34, 139, 34)),
        "gainsboro"            => Some((220, 220, 220)),
        "ghostwhite"           => Some((248, 248, 255)),
        "gold"                 => Some((255, 215, 0)),
        "goldenrod"            => Some((218, 165, 32)),
        "greenyellow"          => Some((173, 255, 47)),
        "grey"                 => Some((128, 128, 128)),
        "honeydew"             => Some((240, 255, 240)),
        "hotpink"              => Some((255, 105, 180)),
        "indianred"            => Some((205, 92, 92)),
        "indigo"               => Some((75, 0, 130)),
        "ivory"                => Some((255, 255, 240)),
        "khaki"                => Some((240, 230, 140)),
        "lavender"             => Some((230, 230, 250)),
        "lavenderblush"        => Some((255, 240, 245)),
        "lawngreen"            => Some((124, 252, 0)),
        "lemonchiffon"         => Some((255, 250, 205)),
        "lightblue"            => Some((173, 216, 230)),
        "lightcoral"           => Some((240, 128, 128)),
        "lightcyan"            => Some((224, 255, 255)),
        "lightgoldenrodyellow" => Some((250, 250, 210)),
        "lightgray"            => Some((211, 211, 211)),
        "lightgreen"           => Some((144, 238, 144)),
        "lightgrey"            => Some((211, 211, 211)),
        "lightpink"            => Some((255, 182, 193)),
        "lightsalmon"          => Some((255, 160, 122)),
        "lightseagreen"        => Some((32, 178, 170)),
        "lightskyblue"         => Some((135, 206, 250)),
        "lightslategray"       => Some((119, 136, 153)),
        "lightslategrey"       => Some((119, 136, 153)),
        "lightsteelblue"       => Some((176, 196, 222)),
        "lightyellow"          => Some((255, 255, 224)),
        "limegreen"            => Some((50, 205, 50)),
        "linen"                => Some((250, 240, 230)),
        "magenta"              => Some((255, 0, 255)),
        "mediumaquamarine"     => Some((102, 205, 170)),
        "mediumblue"           => Some((0, 0, 205)),
        "mediumorchid"         => Some((186, 85, 211)),
        "mediumpurple"         => Some((147, 112, 219)),
        "mediumseagreen"       => Some((60, 179, 113)),
        "mediumslateblue"      => Some((123, 104, 238)),
        "mediumspringgreen"    => Some((0, 250, 154)),
        "mediumturquoise"      => Some((72, 209, 204)),
        "mediumvioletred"      => Some((199, 21, 133)),
        "midnightblue"         => Some((25, 25, 112)),
        "mintcream"            => Some((245, 255, 250)),
        "mistyrose"            => Some((255, 228, 225)),
        "moccasin"             => Some((255, 228, 181)),
        "navajowhite"          => Some((255, 222, 173)),
        "oldlace"              => Some((253, 245, 230)),
        "olivedrab"            => Some((107, 142, 35)),
        "orange"               => Some((255, 165, 0)),
        "orangered"            => Some((255, 69, 0)),
        "orchid"               => Some((218, 112, 214)),
        "palegoldenrod"        => Some((238, 232, 170)),
        "palegreen"            => Some((152, 251, 152)),
        "paleturquoise"        => Some((175, 238, 238)),
        "palevioletred"        => Some((219, 112, 147)),
        "papayawhip"           => Some((255, 239, 213)),
        "peachpuff"            => Some((255, 218, 185)),
        "peru"                 => Some((205, 133, 63)),
        "pink"                 => Some((255, 192, 203)),
        "plum"                 => Some((221, 160, 221)),
        "powderblue"           => Some((176, 224, 230)),
        "rebeccapurple"        => Some((102, 51, 153)),
        "rosybrown"            => Some((188, 143, 143)),
        "royalblue"            => Some((65, 105, 225)),
        "saddlebrown"          => Some((139, 69, 19)),
        "salmon"               => Some((250, 128, 114)),
        "sandybrown"           => Some((244, 164, 96)),
        "seagreen"             => Some((46, 139, 87)),
        "seashell"             => Some((255, 245, 238)),
        "sienna"               => Some((160, 82, 45)),
        "skyblue"              => Some((135, 206, 235)),
        "slateblue"            => Some((106, 90, 205)),
        "slategray"            => Some((112, 128, 144)),
        "slategrey"            => Some((112, 128, 144)),
        "snow"                 => Some((255, 250, 250)),
        "springgreen"          => Some((0, 255, 127)),
        "steelblue"            => Some((70, 130, 180)),
        "tan"                  => Some((210, 180, 140)),
        "thistle"              => Some((216, 191, 216)),
        "tomato"               => Some((255, 99, 71)),
        "turquoise"            => Some((64, 224, 208)),
        "violet"               => Some((238, 130, 238)),
        "wheat"                => Some((245, 222, 179)),
        "whitesmoke"           => Some((245, 245, 245)),
        "yellowgreen"          => Some((154, 205, 50)),
        _ => None,
    }
}

/// 解析 CSS 颜色值
pub fn parse_color(value: &str) -> Option<(u8, u8, u8, u8)> {
    let value = value.trim().to_lowercase();
    
    // 处理渐变 - 提取第一个颜色
    if value.starts_with("linear-gradient(") || value.starts_with("radial-gradient(") {
        let inner = value
            .trim_start_matches("linear-gradient(")
            .trim_start_matches("radial-gradient(")
            .trim_end_matches(')');
        
        // 查找第一个颜色值
        for part in inner.split(',') {
            let part = part.trim();
            // 跳过角度和位置
            if part.starts_with("to ") || part.contains("deg") || part.contains('%') {
                continue;
            }
            // 尝试解析颜色
            if let Some(color) = parse_color(part.split_whitespace().next()?) {
                return Some(color);
            }
        }
        return None;
    }
    
    // 特殊关键字：transparent
    if value == "transparent" {
        return Some((0, 0, 0, 0));
    }
    // currentcolor：由调用方传递继承颜色，这里返回 None 让调用方处理
    if value == "currentcolor" {
        return None;
    }

    // 尝试解析 hsl() / hsla() 格式
    if value.starts_with("hsl(") || value.starts_with("hsla(") {
        return parse_hsl(&value);
    }

    // 尝试解析 rgb(), rgba() 格式（支持逗号分隔和 CSS4 空格分隔）
    if value.starts_with("rgb(") || value.starts_with("rgba(") {
        return parse_rgb(&value);
    }

    // 尝试解析 #RRGGBB 或 #RGB 格式
    if value.starts_with('#') {
        let hex = value.trim_start_matches('#');
        return parse_hex_color(hex);
    }

    // 尝试解析颜色名称
    if let Some((r, g, b)) = color_from_name(&value) {
        return Some((r, g, b, 255));
    }

    None
}

/// 解析 hsl() / hsla() 颜色
fn parse_hsl(value: &str) -> Option<(u8, u8, u8, u8)> {
    let inner = value
        .trim_start_matches("hsl(")
        .trim_start_matches("hsla(")
        .trim_end_matches(')')
        .trim();

    // 支持逗号分隔和空格分隔
    let parts: Vec<&str> = if inner.contains(',') {
        inner.split(',').map(|s| s.trim()).collect()
    } else {
        inner.split_whitespace().collect()
    };

    if parts.len() < 3 {
        return None;
    }

    let h = parse_hue(parts[0])?;
    let s = parse_saturation_lightness(parts[1])?;
    let l = parse_saturation_lightness(parts[2])?;

    let a = if parts.len() >= 4 {
        parse_alpha_component(parts[3]).unwrap_or(1.0)
    } else {
        1.0
    };

    // HSL → RGB 转换
    let (r, g, b) = hsl_to_rgb(h, s, l);
    Some((r, g, b, (a * 255.0) as u8))
}

/// 解析色相值（0-360 度）
fn parse_hue(s: &str) -> Option<f32> {
    let s = s.trim();
    if s.ends_with("deg") {
        s.trim_end_matches("deg").trim().parse().ok()
    } else {
        s.parse().ok()
    }
}

/// 解析饱和度/亮度（百分比）
fn parse_saturation_lightness(s: &str) -> Option<f32> {
    let s = s.trim();
    if s.ends_with('%') {
        let val: f32 = s.trim_end_matches('%').trim().parse().ok()?;
        Some(val / 100.0)
    } else {
        None // HSL 的 s/l 必须带百分号
    }
}

/// HSL → RGB 转换算法
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let h = ((h % 360.0) + 360.0) % 360.0;

    if s == 0.0 {
        let v = (l * 255.0) as u8;
        return (v, v, v);
    }

    let h_norm = h / 60.0;
    let c = if l <= 0.5 { s * (1.0 + l) } else { s * (1.0 + 1.0 - l) };
    let x = c * (1.0 - ((h_norm % 2.0) - 1.0).abs());
    let m = l - if l <= 0.5 { c } else { c };

    let (r1, g1, b1) = match h_norm.floor() as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        5 => (c, 0.0, x),
        _ => (0.0, 0.0, 0.0),
    };

    let r = ((r1 + m) * 255.0).clamp(0.0, 255.0) as u8;
    let g = ((g1 + m) * 255.0).clamp(0.0, 255.0) as u8;
    let b = ((b1 + m) * 255.0).clamp(0.0, 255.0) as u8;
    (r, g, b)
}

/// 解析 rgb() / rgba() 颜色（支持逗号和 CSS4 空格语法）
fn parse_rgb(value: &str) -> Option<(u8, u8, u8, u8)> {
    let inner = value
        .trim_start_matches("rgb(")
        .trim_start_matches("rgba(")
        .trim_end_matches(')')
        .trim();

    // CSS4 语法：rgb(255 0 0 / 0.5) — 用 / 分隔 alpha
    let (color_part, alpha_part) = if let Some(idx) = inner.find('/') {
        let c = inner[..idx].trim();
        let a = inner[idx + 1..].trim();
        (c.to_string(), Some(a.to_string()))
    } else {
        (inner.to_string(), None)
    };

    // 尝试空格分隔（CSS4），回退到逗号分隔（CSS3）
    let parts: Vec<&str> = if color_part.contains(' ') && !color_part.contains(',') {
        color_part.split_whitespace().collect()
    } else {
        color_part.split(',').map(|s| s.trim()).collect()
    };

    if parts.len() < 3 {
        return None;
    }

    let r = parse_color_component(parts[0]).ok()?;
    let g = parse_color_component(parts[1]).ok()?;
    let b = parse_color_component(parts[2]).ok()?;

    let a = match alpha_part {
        Some(s) => parse_alpha_component(&s).unwrap_or(1.0),
        None => {
            if parts.len() >= 4 {
                parse_alpha_component(parts[3]).unwrap_or(1.0)
            } else {
                1.0
            }
        }
    };

    Some((r, g, b, (a * 255.0) as u8))
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
    /// 视口相对单位
    Vw(f32),
    Vh(f32),
    Vmin(f32),
    Vmax(f32),
    /// 字体相对单位
    Ch(f32),
    Ex(f32),
    /// 绝对单位
    Pt(f32),
    Pc(f32),
    Cm(f32),
    Mm(f32),
    In(f32),
    Auto,
}

impl Length {
    pub fn parse(value: &str) -> Option<Self> {
        let value = value.trim();

        if value == "auto" {
            return Some(Length::Auto);
        }

        // rem 必须在 em 之前检查！"1rem".ends_with("em") 为 true
        if value.ends_with("rem") {
            let num: f32 = value.trim_end_matches("rem").parse().ok()?;
            return Some(Length::Rem(num));
        } else if value.ends_with("px") {
            let num: f32 = value.trim_end_matches("px").parse().ok()?;
            Some(Length::Px(num))
        } else if value.ends_with("vh") {
            let num: f32 = value.trim_end_matches("vh").parse().ok()?;
            Some(Length::Vh(num))
        } else if value.ends_with("vw") {
            let num: f32 = value.trim_end_matches("vw").parse().ok()?;
            Some(Length::Vw(num))
        } else if value.ends_with("vmin") {
            let num: f32 = value.trim_end_matches("vmin").parse().ok()?;
            Some(Length::Vmin(num))
        } else if value.ends_with("vmax") {
            let num: f32 = value.trim_end_matches("vmax").parse().ok()?;
            Some(Length::Vmax(num))
        } else if value.ends_with("ch") {
            let num: f32 = value.trim_end_matches("ch").parse().ok()?;
            Some(Length::Ch(num))
        } else if value.ends_with("ex") {
            let num: f32 = value.trim_end_matches("ex").parse().ok()?;
            Some(Length::Ex(num))
        } else if value.ends_with("em") {
            let num: f32 = value.trim_end_matches("em").parse().ok()?;
            Some(Length::Em(num))
        } else if value.ends_with("pt") {
            let num: f32 = value.trim_end_matches("pt").parse().ok()?;
            Some(Length::Pt(num))
        } else if value.ends_with("pc") {
            let num: f32 = value.trim_end_matches("pc").parse().ok()?;
            Some(Length::Pc(num))
        } else if value.ends_with("cm") {
            let num: f32 = value.trim_end_matches("cm").parse().ok()?;
            Some(Length::Cm(num))
        } else if value.ends_with("mm") {
            let num: f32 = value.trim_end_matches("mm").parse().ok()?;
            Some(Length::Mm(num))
        } else if value.ends_with("in") {
            let num: f32 = value.trim_end_matches("in").parse().ok()?;
            Some(Length::In(num))
        } else if value.ends_with('%') {
            let num: f32 = value.trim_end_matches('%').parse().ok()?;
            Some(Length::Percent(num))
        } else {
            // 假设是像素
            let num: f32 = value.parse().ok()?;
            Some(Length::Px(num))
        }
    }

    /// 转换为像素值（需要父元素尺寸和字体上下文）
    pub fn to_px(&self, parent_value: f32, font_size: f32, viewport_width: f32, viewport_height: f32) -> f32 {
        match self {
            Length::Px(v) => *v,
            Length::Em(v) => *v * font_size,
            Length::Rem(v) => *v * font_size,
            Length::Percent(v) => *v / 100.0 * parent_value,
            Length::Vw(v) => *v / 100.0 * viewport_width,
            Length::Vh(v) => *v / 100.0 * viewport_height,
            Length::Vmin(v) => *v / 100.0 * viewport_width.min(viewport_height),
            Length::Vmax(v) => *v / 100.0 * viewport_width.max(viewport_height),
            Length::Ch(v) => *v * font_size * 0.5, // ch ≈ 0.5em
            Length::Ex(v) => *v * font_size * 0.5, // ex ≈ 0.5em
            Length::Pt(v) => *v * 1.333,          // 1pt = 1.333px
            Length::Pc(v) => *v * 16.0,           // 1pc = 16px
            Length::Cm(v) => *v * 37.8,           // 1cm ≈ 37.8px
            Length::Mm(v) => *v * 3.78,           // 1mm ≈ 3.78px
            Length::In(v) => *v * 96.0,           // 1in = 96px
            Length::Auto => 0.0,
        }
    }
}
