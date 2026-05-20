//! CSS Transform 解析与计算
//! 
//! 支持 2D 变换：translate, rotate, scale, skew, matrix

/// CSS 变换函数
#[derive(Debug, Clone, PartialEq)]
pub enum TransformFunction {
    /// 平移: translate(tx, ty) 或 translate(tx)
    Translate { tx: f32, ty: Option<f32> },
    /// 缩放: scale(sx, sy) 或 scale(s)
    Scale { sx: f32, sy: Option<f32> },
    /// 旋转: rotate(angle)
    Rotate { angle: f32 }, // 角度（度）
    /// 倾斜: skew(ax, ay) 或 skew(ax)
    Skew { ax: f32, ay: Option<f32> },
    /// 矩阵变换: matrix(a, b, c, d, tx, ty)
    Matrix { a: f32, b: f32, c: f32, d: f32, tx: f32, ty: f32 },
}

impl TransformFunction {
    /// 解析变换函数字符串
    pub fn parse(transform_str: &str) -> Option<Vec<TransformFunction>> {
        let transform_str = transform_str.trim();
        if transform_str.is_empty() || transform_str == "none" {
            return Some(vec![]);
        }

        let mut transforms = Vec::new();
        let mut pos = 0;
        let bytes = transform_str.as_bytes();

        while pos < transform_str.len() {
            // 跳过空白
            while pos < transform_str.len() && bytes[pos].is_ascii_whitespace() {
                pos += 1;
            }
            if pos >= transform_str.len() {
                break;
            }

            // 查找函数名
            let func_start = pos;
            while pos < transform_str.len() && bytes[pos] != b'(' {
                pos += 1;
            }
            if pos >= transform_str.len() {
                break;
            }

            let func_name = &transform_str[func_start..pos].to_lowercase();
            pos += 1; // 跳过 (

            // 查找匹配的 )
            let args_start = pos;
            let mut depth = 1;
            while pos < transform_str.len() && depth > 0 {
                match bytes[pos] {
                    b'(' => depth += 1,
                    b')' => depth -= 1,
                    _ => {}
                }
                if depth > 0 {
                    pos += 1;
                }
            }
            if pos >= transform_str.len() {
                break;
            }

            let args_str = &transform_str[args_start..pos];
            pos += 1; // 跳过 )

            // 解析具体变换函数
            if let Some(transform) = parse_transform_function(func_name, args_str) {
                transforms.push(transform);
            }
        }

        if transforms.is_empty() {
            None
        } else {
            Some(transforms)
        }
    }
}

/// 解析单个变换函数
fn parse_transform_function(name: &str, args: &str) -> Option<TransformFunction> {
    let args: Vec<&str> = args.split(',').map(|s| s.trim()).collect();

    match name {
        "translate" => {
            let tx = parse_length(args[0])?;
            let ty = if args.len() > 1 {
                Some(parse_length(args[1])?)
            } else {
                None
            };
            Some(TransformFunction::Translate { tx, ty })
        }
        "translateX" => {
            let tx = parse_length(args[0])?;
            Some(TransformFunction::Translate { tx, ty: Some(0.0) })
        }
        "translateY" => {
            let ty = parse_length(args[0])?;
            Some(TransformFunction::Translate { tx: 0.0, ty: Some(ty) })
        }
        "scale" => {
            let sx = args[0].parse::<f32>().ok()?;
            let sy = if args.len() > 1 {
                Some(args[1].parse::<f32>().ok()?)
            } else {
                None
            };
            Some(TransformFunction::Scale { sx, sy })
        }
        "scaleX" => {
            let sx = args[0].parse::<f32>().ok()?;
            Some(TransformFunction::Scale { sx, sy: Some(1.0) })
        }
        "scaleY" => {
            let sy = args[0].parse::<f32>().ok()?;
            Some(TransformFunction::Scale { sx: 1.0, sy: Some(sy) })
        }
        "rotate" => {
            let angle = parse_angle(args[0])?;
            Some(TransformFunction::Rotate { angle })
        }
        "skew" => {
            let ax = parse_angle(args[0])?;
            let ay = if args.len() > 1 {
                Some(parse_angle(args[1])?)
            } else {
                None
            };
            Some(TransformFunction::Skew { ax, ay })
        }
        "skewX" => {
            let ax = parse_angle(args[0])?;
            Some(TransformFunction::Skew { ax, ay: Some(0.0) })
        }
        "skewY" => {
            let ay = parse_angle(args[0])?;
            Some(TransformFunction::Skew { ax: 0.0, ay: Some(ay) })
        }
        "matrix" => {
            if args.len() != 6 {
                return None;
            }
            Some(TransformFunction::Matrix {
                a: args[0].parse::<f32>().ok()?,
                b: args[1].parse::<f32>().ok()?,
                c: args[2].parse::<f32>().ok()?,
                d: args[3].parse::<f32>().ok()?,
                tx: args[4].parse::<f32>().ok()?,
                ty: args[5].parse::<f32>().ok()?,
            })
        }
        _ => None,
    }
}

/// 解析长度值（支持 px 和百分比）
fn parse_length(value: &str) -> Option<f32> {
    let value = value.trim();
    
    if value.ends_with("px") {
        value.trim_end_matches("px").trim().parse::<f32>().ok()
    } else if value.ends_with('%') {
        // 百分比需要根据上下文转换，这里简化处理
        value.trim_end_matches('%').trim().parse::<f32>().ok()
    } else {
        // 默认为 px
        value.parse::<f32>().ok()
    }
}

/// 解析角度值（支持 deg, rad, turn）
fn parse_angle(value: &str) -> Option<f32> {
    let value = value.trim();
    
    if value.ends_with("deg") {
        value.trim_end_matches("deg").trim().parse::<f32>().ok()
    } else if value.ends_with("rad") {
        let rad = value.trim_end_matches("rad").trim().parse::<f32>().ok()?;
        Some(rad * 180.0 / std::f32::consts::PI) // 转换为度
    } else if value.ends_with("turn") {
        let turn = value.trim_end_matches("turn").trim().parse::<f32>().ok()?;
        Some(turn * 360.0)
    } else {
        // 默认为度
        value.parse::<f32>().ok()
    }
}

/// 将变换列表转换为变换矩阵
/// 
/// 返回一个包含6个元素的数组，对应 matrix(a, b, c, d, tx, ty)
/// 调用方可以将其转换为 tiny_skia::Transform 或其他渲染系统的变换矩阵
pub fn transforms_to_matrix(transforms: &[TransformFunction], element_width: f32, element_height: f32) -> [f32; 6] {
    // 初始为单位矩阵
    let mut matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]; // [a, b, c, d, tx, ty]
    
    for tf in transforms {
        matrix = match tf {
            TransformFunction::Translate { tx, ty } => {
                let ty_val = ty.unwrap_or(0.0);
                multiply_matrices(&matrix, &translation_matrix(*tx, ty_val))
            }
            TransformFunction::Scale { sx, sy } => {
                let sy_val = sy.unwrap_or(*sx);
                multiply_matrices(&matrix, &scale_matrix(*sx, sy_val))
            }
            TransformFunction::Rotate { angle } => {
                // 围绕元素中心旋转
                let cx = element_width / 2.0;
                let cy = element_height / 2.0;
                let rad = angle.to_radians();
                
                // translate(cx, cy) * rotate(angle) * translate(-cx, -cy)
                let t1 = translation_matrix(cx, cy);
                let r = rotation_matrix(rad);
                let t2 = translation_matrix(-cx, -cy);
                let combined = multiply_matrices(&t1, &multiply_matrices(&r, &t2));
                multiply_matrices(&matrix, &combined)
            }
            TransformFunction::Skew { ax, ay } => {
                let ay_val = ay.unwrap_or(0.0);
                let ax_rad = ax.to_radians();
                let ay_rad = ay_val.to_radians();
                multiply_matrices(&matrix, &skew_matrix(ax_rad, ay_rad))
            }
            TransformFunction::Matrix { a, b, c, d, tx, ty } => {
                multiply_matrices(&matrix, &[*a, *b, *c, *d, *tx, *ty])
            }
        };
    }
    
    matrix
}

/// 创建平移矩阵
fn translation_matrix(tx: f32, ty: f32) -> [f32; 6] {
    [1.0, 0.0, 0.0, 1.0, tx, ty]
}

/// 创建缩放矩阵
fn scale_matrix(sx: f32, sy: f32) -> [f32; 6] {
    [sx, 0.0, 0.0, sy, 0.0, 0.0]
}

/// 创建旋转矩阵
fn rotation_matrix(rad: f32) -> [f32; 6] {
    let cos = rad.cos();
    let sin = rad.sin();
    [cos, sin, -sin, cos, 0.0, 0.0]
}

/// 创建倾斜矩阵
fn skew_matrix(ax: f32, ay: f32) -> [f32; 6] {
    [1.0, ay.tan(), ax.tan(), 1.0, 0.0, 0.0]
}

/// 矩阵乘法（2D仿射变换）
fn multiply_matrices(m1: &[f32; 6], m2: &[f32; 6]) -> [f32; 6] {
    [
        m1[0] * m2[0] + m1[2] * m2[1], // a
        m1[1] * m2[0] + m1[3] * m2[1], // b
        m1[0] * m2[2] + m1[2] * m2[3], // c
        m1[1] * m2[2] + m1[3] * m2[3], // d
        m1[0] * m2[4] + m1[2] * m2[5] + m1[4], // tx
        m1[1] * m2[4] + m1[3] * m2[5] + m1[5], // ty
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_translate() {
        let transforms = TransformFunction::parse("translate(10px, 20px)").unwrap();
        assert_eq!(transforms.len(), 1);
        match &transforms[0] {
            TransformFunction::Translate { tx, ty } => {
                assert_eq!(*tx, 10.0);
                assert_eq!(*ty, Some(20.0));
            }
            _ => panic!("Expected Translate"),
        }
    }

    #[test]
    fn test_parse_rotate() {
        let transforms = TransformFunction::parse("rotate(45deg)").unwrap();
        assert_eq!(transforms.len(), 1);
        match &transforms[0] {
            TransformFunction::Rotate { angle } => {
                assert_eq!(*angle, 45.0);
            }
            _ => panic!("Expected Rotate"),
        }
    }

    #[test]
    fn test_parse_multiple() {
        let transforms = TransformFunction::parse("translate(10px, 20px) rotate(45deg) scale(1.5)").unwrap();
        assert_eq!(transforms.len(), 3);
    }
}
