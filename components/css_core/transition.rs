//! CSS Transition 解析
//! 
//! 支持 transition 属性的解析和状态计算

/// 单个过渡属性定义
#[derive(Debug, Clone)]
pub struct Transition {
    /// 过渡属性名（如 "opacity", "transform", "all"）
    pub property: String,
    /// 过渡持续时间（秒）
    pub duration: f32,
    /// 过渡时间函数（如 "ease", "linear", "ease-in", "ease-out", "ease-in-out"）
    pub timing_function: String,
    /// 过渡延迟（秒）
    pub delay: f32,
}

impl Transition {
    /// 解析 transition 属性值
    /// 
    /// 支持格式：
    /// - `transition: all 0.3s ease`
    /// - `transition: opacity 0.5s linear 0.1s`
    /// - `transition: width 0.3s ease-in, height 0.3s ease-out`
    pub fn parse(value: &str) -> Vec<Transition> {
        let mut transitions = Vec::new();
        
        // 多个过渡用逗号分隔
        for transition_str in value.split(',') {
            if let Some(transition) = parse_single_transition(transition_str.trim()) {
                transitions.push(transition);
            }
        }
        
        if transitions.is_empty() {
            // 默认过渡
            transitions.push(Transition {
                property: "all".to_string(),
                duration: 0.0,
                timing_function: "ease".to_string(),
                delay: 0.0,
            });
        }
        
        transitions
    }
    
    /// 计算过渡进度（0.0 - 1.0）
    /// 
    /// 参数：
    /// - elapsed: 已经过的时间（秒）
    /// - 返回：过渡进度（0.0表示开始，1.0表示完成）
    pub fn calculate_progress(&self, elapsed: f32) -> f32 {
        if self.duration <= 0.0 {
            return 1.0; // 无过渡，立即完成
        }
        
        // 考虑延迟
        let adjusted_elapsed = (elapsed - self.delay).max(0.0);
        let progress = (adjusted_elapsed / self.duration).clamp(0.0, 1.0);
        
        // 应用时间函数
        apply_timing_function(&self.timing_function, progress)
    }
    
    /// 计算插值
    /// 
    /// 在起始值和结束值之间根据进度进行插值
    pub fn interpolate(&self, start: f32, end: f32, elapsed: f32) -> f32 {
        let progress = self.calculate_progress(elapsed);
        start + (end - start) * progress
    }
}

/// 解析单个过渡定义
fn parse_single_transition(value: &str) -> Option<Transition> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    
    if parts.is_empty() {
        return None;
    }
    
    let property = parts[0].to_string();
    let mut duration = 0.0;
    let mut timing_function = "ease".to_string();
    let mut delay = 0.0;
    
    let mut time_values_found = 0;
    
    for part in &parts[1..] {
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
        } else {
            // 可能是时间函数
            timing_function = part.to_string();
        }
    }
    
    Some(Transition {
        property,
        duration,
        timing_function,
        delay,
    })
}

/// 解析时间值（支持 s 和 ms）
fn parse_time(value: &str) -> Option<f32> {
    let value = value.trim();
    
    if value.ends_with("ms") {
        let ms = value.trim_end_matches("ms").trim().parse::<f32>().ok()?;
        Some(ms / 1000.0) // 转换为秒
    } else if value.ends_with('s') {
        value.trim_end_matches('s').trim().parse::<f32>().ok()
    } else {
        None
    }
}

/// 应用时间函数
fn apply_timing_function(timing_function: &str, progress: f32) -> f32 {
    match timing_function {
        "linear" => progress,
        "ease" => {
            // cubic-bezier(0.25, 0.1, 0.25, 1.0)
            cubic_bezier(progress, 0.25, 0.1, 0.25, 1.0)
        }
        "ease-in" => {
            // cubic-bezier(0.42, 0, 1.0, 1.0)
            cubic_bezier(progress, 0.42, 0.0, 1.0, 1.0)
        }
        "ease-out" => {
            // cubic-bezier(0, 0, 0.58, 1.0)
            cubic_bezier(progress, 0.0, 0.0, 0.58, 1.0)
        }
        "ease-in-out" => {
            // cubic-bezier(0.42, 0, 0.58, 1.0)
            cubic_bezier(progress, 0.42, 0.0, 0.58, 1.0)
        }
        _ => {
            // 尝试解析 cubic-bezier
            if let Some(bezier) = parse_cubic_bezier(timing_function) {
                cubic_bezier(progress, bezier.0, bezier.1, bezier.2, bezier.3)
            } else {
                progress // 默认 linear
            }
        }
    }
}

/// 解析 cubic-bezier 函数
fn parse_cubic_bezier(value: &str) -> Option<(f32, f32, f32, f32)> {
    if !value.starts_with("cubic-bezier(") || !value.ends_with(')') {
        return None;
    }
    
    let inner = &value[13..value.len()-1];
    let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
    
    if parts.len() != 4 {
        return None;
    }
    
    Some((
        parts[0].parse::<f32>().ok()?,
        parts[1].parse::<f32>().ok()?,
        parts[2].parse::<f32>().ok()?,
        parts[3].parse::<f32>().ok()?,
    ))
}

/// 计算三次贝塞尔曲线
fn cubic_bezier(t: f32, p1x: f32, p1y: f32, p2x: f32, p2y: f32) -> f32 {
    // 简化的贝塞尔曲线计算（使用二分法）
    let mut t_lower = 0.0;
    let mut t_upper = 1.0;
    let mut t_mid = t;
    
    // 迭代10次足够精确
    for _ in 0..10 {
        let x = bezier_x(t_mid, p1x, p2x);
        if (x - t).abs() < 0.001 {
            break;
        }
        if x < t {
            t_lower = t_mid;
        } else {
            t_upper = t_mid;
        }
        t_mid = (t_lower + t_upper) / 2.0;
    }
    
    bezier_y(t_mid, p1y, p2y)
}

/// 贝塞尔曲线 X 坐标
fn bezier_x(t: f32, p1x: f32, p2x: f32) -> f32 {
    let u = 1.0 - t;
    3.0 * u * u * t * p1x + 3.0 * u * t * t * p2x + t * t * t
}

/// 贝塞尔曲线 Y 坐标
fn bezier_y(t: f32, p1y: f32, p2y: f32) -> f32 {
    let u = 1.0 - t;
    3.0 * u * u * t * p1y + 3.0 * u * t * t * p2y + t * t * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_transition() {
        let transitions = Transition::parse("opacity 0.3s ease");
        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].property, "opacity");
        assert_eq!(transitions[0].duration, 0.3);
        assert_eq!(transitions[0].timing_function, "ease");
    }

    #[test]
    fn test_parse_multiple_transitions() {
        let transitions = Transition::parse("width 0.3s ease, height 0.5s linear");
        assert_eq!(transitions.len(), 2);
        assert_eq!(transitions[0].property, "width");
        assert_eq!(transitions[1].property, "height");
    }

    #[test]
    fn test_parse_with_delay() {
        let transitions = Transition::parse("opacity 0.3s ease 0.1s");
        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].delay, 0.1);
    }

    #[test]
    fn test_progress_calculation() {
        let transition = Transition {
            property: "opacity".to_string(),
            duration: 1.0,
            timing_function: "linear".to_string(),
            delay: 0.0,
        };
        
        assert_eq!(transition.calculate_progress(0.0), 0.0);
        assert_eq!(transition.calculate_progress(0.5), 0.5);
        assert_eq!(transition.calculate_progress(1.0), 1.0);
        assert_eq!(transition.calculate_progress(2.0), 1.0); // 超过1.0时钳制
    }
}
