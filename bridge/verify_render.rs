//! 验证渲染质量

use servo_bridge::{RealServoBridge, WebNativeBridge};

fn main() {
    println!("=== 渲染质量验证 ===\n");

    // 创建桥接器
    let mut bridge = RealServoBridge::new(400, 300);

    // 测试HTML
    let html = r#"
        <div id="parent" style="background-color: #3498db; width: 300px; height: 200px;">
            <div id="child" style="background-color: #e74c3c; width: 150px; height: 80px;">
                Child Text
            </div>
        </div>
    "#;

    println!("1. 设置HTML...");
    bridge.set_html(html);

    println!("\n2. 检查布局节点:");
    let rects = bridge.all_rects();
    println!("   节点数量: {}", rects.len());
    
    if rects.len() != 2 {
        println!("   ❌ 错误: 应该有2个节点，实际有 {}", rects.len());
    } else {
        println!("   ✓ 节点数量正确");
    }

    for (i, rect) in rects.iter().enumerate() {
        println!("\n   节点 {}:", i);
        println!("     标签: {}", rect.tag_name);
        println!("     位置: ({:.1}, {:.1})", rect.x, rect.y);
        println!("     尺寸: {:.1}x{:.1}", rect.width, rect.height);
        
        if let Some(color) = &rect.background {
            println!("     背景色: RGB({}, {}, {})", color.r, color.g, color.b);
            
            // 验证颜色
            if rect.tag_name == "div" && rect.width > 200.0 {
                // 父div应该是蓝色 #3498db = RGB(52, 152, 219)
                if color.r == 52 && color.g == 152 && color.b == 219 {
                    println!("     ✓ 父div颜色正确");
                } else {
                    println!("     ❌ 父div颜色错误，期望(52, 152, 219)");
                }
            } else if rect.tag_name == "div" && rect.width <= 200.0 {
                // 子div应该是红色 #e74c3c = RGB(231, 76, 60)
                if color.r == 231 && color.g == 76 && color.b == 60 {
                    println!("     ✓ 子div颜色正确");
                } else {
                    println!("     ❌ 子div颜色错误，期望(231, 76, 60)");
                }
            }
        } else {
            println!("     ❌ 没有背景色");
        }
    }

    println!("\n3. 验证布局关系:");
    if rects.len() == 2 {
        // rects[0]是子div, rects[1]是父div
        let child = &rects[0];
        let parent = &rects[1];
        
        // 子元素应该在父元素内部（考虑padding偏移）
        if child.x >= parent.x && child.y >= parent.y {
            println!("   ✓ 子元素在父元素内部 (偏移{:.0}, {:.0})", child.x - parent.x, child.y - parent.y);
        } else {
            println!("   ❌ 子元素位置错误");
        }
        
        // 子元素应该比父元素小
        if child.width < parent.width && child.height < parent.height {
            println!("   ✓ 子元素尺寸正确 ({}x{} < {}x{})", child.width, child.height, parent.width, parent.height);
        } else {
            println!("   ❌ 子元素尺寸错误");
        }
    }

    println!("\n4. 渲染PNG...");
    let png_data = bridge.render();
    
    if png_data.len() > 10000 {
        println!("   ✓ PNG大小合理: {} bytes", png_data.len());
    } else {
        println!("   ❌ PNG太小: {} bytes，可能渲染失败", png_data.len());
    }

    // 保存
    let output_path = "verify_output.png";
    match std::fs::write(output_path, &png_data) {
        Ok(_) => {
            println!("\n5. ✓ PNG已保存到: {}", output_path);
            println!("   请手动打开查看渲染效果！");
        }
        Err(e) => {
            println!("\n5. ❌ 保存失败: {}", e);
        }
    }

    println!("\n=== 验证完成 ===");
    println!("\n请检查:");
    println!("1. verify_output.png 是否有两个矩形？");
    println!("2. 大矩形是蓝色吗？");
    println!("3. 小矩形是红色吗？");
    println!("4. 小矩形在大矩形内部吗？");
    println!("5. 能看到文本 'Child Text' 吗？");
}
