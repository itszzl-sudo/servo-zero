//! CSS Animation 解析
//! 
//! 支持 @keyframes 和 animation 属性解析

/// 关键帧定义
#[derive(Debug, Clone)]
pub struct Keyframe {
    /// 关键帧位置（0.0 - 1.0，或使用 from/to）
    pub offset: f32,
    /// 该关键帧的CSS属性值
    pub properties: Vec<(String, String)>,
}

/// @keyframes 动画定义
#[derive(Debug, Clone)]
pub struct KeyframesAnimation {
    /// 动画名称
    pub name: String,
    /// 关键帧列表
    pub keyframes: Vec<Keyframe>,
}

/// animation 属性定义
#[derive(Debug, Clone)]
pub struct Animation {
    /// 动画名称（引用 @keyframes）
    pub name: String,
    /// 动画持续时间（秒）
    pub duration: f32,
    /// 时间函数
    pub timing_function: String,
    /// 延迟时间（秒）
    pub delay: f32,
    /// 迭代次数（None表示无限循环）
    pub iteration_count: Option<f32>,
    /// 播放方向（normal, reverse, alternate, alternate-reverse）
    pub direction: String,
    /// 填充模式（none, forwards, backwards, both）
    pub fill_mode: String,
    /// 播放状态（running, paused）
    pub play_state: String,
}

impl Animation {
    /// 解析 animation 属性值
    /// 
    /// 支持格式：
    /// - `animation: slidein 3s ease`
    /// - `animation: slidein 3s ease 1s 2 alternate`
    pub fn parse(value: &str) -> Vec<Animation> {
        let mut animations = Vec::new();
        
        // 多个动画用逗号分隔
        for anim_str in value.split(',') {
            if let Some(animation) = parse_single_animation(anim_str.trim()) {
                animations.push(animation);
            }
        }
        
        animations
    }
    
    /// 计算动画当前状态
    /// 
    /// 返回：(当前关键帧偏移量 0.0-1.0, 当前迭代次数)
    pub fn calculate_state(&self, elapsed: f32) -> (f32, f32) {
        if self.duration <= 0.0 {
            return (0.0, 0.0);
        }
        
        // 考虑延迟
        let adjusted_elapsed = (elapsed - self.delay).max(0.0);
        
        // 计算当前迭代次数
        let total_iterations = adjusted_elapsed / self.duration;
        let current_iteration = total_iterations.floor();
        let progress_in_iteration = total_iterations - current_iteration;
        
        // 处理迭代次数限制
        if let Some(max_iterations) = self.iteration_count {
            if current_iteration >= max_iterations {
                // 动画已结束
                let final_progress = if self.fill_mode == "forwards" || self.fill_mode == "both" {
                    1.0
                } else {
                    0.0
                };
                return (final_progress, max_iterations);
            }
        }
        
        // 处理播放方向
        let mut progress = progress_in_iteration;
        if self.direction == "reverse" || self.direction == "alternate-reverse" {
            progress = 1.0 - progress;
        } else if self.direction == "alternate" && current_iteration as i32 % 2 == 1 {
            progress = 1.0 - progress;
        }
        
        (progress.clamp(0.0, 1.0), current_iteration)
    }
    
    /// 从关键帧中插值获取属性值
    pub fn interpolate_property(
        &self,
        keyframes: &KeyframesAnimation,
        property: &str,
        elapsed: f32,
    ) -> Option<String> {
        let (progress, _) = self.calculate_state(elapsed);
        
        // 找到进度前后两个关键帧
        let mut prev_keyframe: Option<&Keyframe> = None;
        let mut next_keyframe: Option<&Keyframe> = None;
        
        for kf in &keyframes.keyframes {
            if kf.offset <= progress {
                prev_keyframe = Some(kf);
            }
            if kf.offset >= progress && next_keyframe.is_none() {
                next_keyframe = Some(kf);
            }
        }
        
        // 处理边界情况
        match (prev_keyframe, next_keyframe) {
            (Some(prev), Some(next)) if prev.offset != next.offset => {
                // 在两个关键帧之间插值
                let local_progress = (progress - prev.offset) / (next.offset - prev.offset);
                interpolate_property_value(property, prev, next, local_progress)
            }
            (Some(kf), _) | (_, Some(kf)) => {
                // 使用最近的关键帧
                kf.properties.iter()
                    .find(|(p, _)| p == property)
                    .map(|(_, v)| v.clone())
            }
            (None, None) => None,
        }
    }
}

/// 解析单个动画定义
fn parse_single_animation(value: &str) -> Option<Animation> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    
    if parts.is_empty() {
        return None;
    }
    
    let mut name = String::new();
    let mut duration = 0.0;
    let mut timing_function = "ease".to_string();
    let mut delay = 0.0;
    let mut iteration_count: Option<f32> = None;
    let mut direction = "normal".to_string();
    let mut fill_mode = "none".to_string();
    let mut play_state = "running".to_string();
    
    let mut time_values_found = 0;
    
    for part in &parts {
        let part = part.trim();
        
        // 尝试解析为时间值
        if let Some(time) = parse_time(part) {
            if time_values_found == 0 {
                duration = time;
                time_values_found += 1;
            } else if time_values_found == 1 {
                delay = time;
                time_values_found += 1;
            }
        } else if let Ok(count) = part.parse::<f32>() {
            iteration_count = Some(count);
        } else {
            match part {
                "infinite" => iteration_count = None,
                "normal" | "reverse" | "alternate" | "alternate-reverse" => {
                    direction = part.to_string();
                }
                "none" | "forwards" | "backwards" | "both" => {
                    fill_mode = part.to_string();
                }
                "running" | "paused" => {
                    play_state = part.to_string();
                }
                "ease" | "linear" | "ease-in" | "ease-out" | "ease-in-out" => {
                    timing_function = part.to_string();
                }
                _ if part.starts_with("cubic-bezier(") => {
                    timing_function = part.to_string();
                }
                _ if name.is_empty() => {
                    name = part.to_string();
                }
                _ => {} // 忽略未知值
            }
        }
    }
    
    if name.is_empty() {
        None
    } else {
        Some(Animation {
            name,
            duration,
            timing_function,
            delay,
            iteration_count,
            direction,
            fill_mode,
            play_state,
        })
    }
}

/// 解析时间值
fn parse_time(value: &str) -> Option<f32> {
    let value = value.trim();
    
    if value.ends_with("ms") {
        let ms = value.trim_end_matches("ms").trim().parse::<f32>().ok()?;
        Some(ms / 1000.0)
    } else if value.ends_with('s') {
        value.trim_end_matches('s').trim().parse::<f32>().ok()
    } else {
        None
    }
}

/// 解析 @keyframes 规则
pub fn parse_keyframes(css: &str) -> Option<KeyframesAnimation> {
    let css = css.trim();
    
    // 匹配 @keyframes name { ... }
    if !css.starts_with("@keyframes") {
        return None;
    }
    
    // 提取名称
    let name_start = 11; // "@keyframes".len()
    let brace_pos = css.find('{')?;
    let name = css[name_start..brace_pos].trim().to_string();
    
    // 提取内容
    let content_start = brace_pos + 1;
    let content_end = css.rfind('}')?;
    let content = &css[content_start..content_end];
    
    let keyframes = parse_keyframe_content(content);
    
    Some(KeyframesAnimation { name, keyframes })
}

/// 解析关键帧内容
fn parse_keyframe_content(content: &str) -> Vec<Keyframe> {
    let mut keyframes = Vec::new();
    
    // 按 } 分割各个关键帧块
    for block in content.split('}') {
        let block = block.trim();
        if block.is_empty() || !block.contains('{') {
            continue;
        }
        
        if let Some(brace_pos) = block.find('{') {
            let selector = &block[..brace_pos].trim();
            let properties_str = &block[brace_pos + 1..];
            
            // 解析选择器（可能是百分比或 from/to）
            let offset = parse_keyframe_selector(selector);
            
            // 解析属性
            let properties = parse_keyframe_properties(properties_str);
            
            keyframes.push(Keyframe { offset, properties });
        }
    }
    
    // 按 offset 排序
    keyframes.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());
    
    keyframes
}

/// 解析关键帧选择器
fn parse_keyframe_selector(selector: &str) -> f32 {
    match selector {
        "from" => 0.0,
        "to" => 1.0,
        _ => {
            // 解析百分比
            if selector.ends_with('%') {
                selector.trim_end_matches('%').trim().parse::<f32>().ok().unwrap_or(0.0) / 100.0
            } else {
                0.0
            }
        }
    }
}

/// 解析关键帧属性
fn parse_keyframe_properties(properties_str: &str) -> Vec<(String, String)> {
    let mut properties = Vec::new();
    
    for decl in properties_str.split(';') {
        let decl = decl.trim();
        if let Some(colon_pos) = decl.find(':') {
            let prop = decl[..colon_pos].trim().to_string();
            let value = decl[colon_pos + 1..].trim().to_string();
            if !prop.is_empty() && !value.is_empty() {
                properties.push((prop, value));
            }
        }
    }
    
    properties
}

/// 插值属性值（简化版，仅支持数值）
fn interpolate_property_value(
    property: &str,
    prev: &Keyframe,
    next: &Keyframe,
    progress: f32,
) -> Option<String> {
    let prev_value = prev.properties.iter().find(|(p, _)| p == property)?.1.parse::<f32>().ok()?;
    let next_value = next.properties.iter().find(|(p, _)| p == property)?.1.parse::<f32>().ok()?;
    
    let interpolated = prev_value + (next_value - prev_value) * progress;
    Some(format!("{}", interpolated))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_animation() {
        let animations = Animation::parse("slidein 3s ease 1s 2 alternate");
        assert_eq!(animations.len(), 1);
        assert_eq!(animations[0].name, "slidein");
        assert_eq!(animations[0].duration, 3.0);
        assert_eq!(animations[0].iteration_count, Some(2.0));
        assert_eq!(animations[0].direction, "alternate");
    }

    #[test]
    fn test_parse_keyframes() {
        let css = r#"
            @keyframes slidein {
                from { transform: translateX(0px); }
                to { transform: translateX(100px); }
            }
        "#;
        
        let keyframes = parse_keyframes(css).unwrap();
        assert_eq!(keyframes.name, "slidein");
        assert_eq!(keyframes.keyframes.len(), 2);
        assert_eq!(keyframes.keyframes[0].offset, 0.0);
        assert_eq!(keyframes.keyframes[1].offset, 1.0);
    }

    #[test]
    fn test_animation_state() {
        let animation = Animation {
            name: "test".to_string(),
            duration: 2.0,
            timing_function: "linear".to_string(),
            delay: 0.0,
            iteration_count: None,
            direction: "normal".to_string(),
            fill_mode: "none".to_string(),
            play_state: "running".to_string(),
        };
        
        let (progress, iteration) = animation.calculate_state(1.0);
        assert_eq!(progress, 0.5);
        assert_eq!(iteration, 0.0);
    }
}
