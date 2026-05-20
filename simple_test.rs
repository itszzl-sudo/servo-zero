//! 简单测试渲染

use servo_bridge::{RealServoBridge, WebNativeBridge};

fn main() {
    println!("=== 简单渲染测试 ===\n");

    // 创建桥接器
    let mut bridge = RealServoBridge::new(400, 300);
    println!("✓ 创建桥接器: 400x300");

    // 设置非常简单的HTML
    let html = r#"
        <div id="box1" style="background: red; width: 100px; height: 50px;">
            Box 1
        </div>
        <div id="box2" style="background: blue; width: 150px; height: 80px;">
            Box 2
        </div>
    "#;

    println!("\n设置HTML内容...");
    bridge.set_html(html);
    println!("✓ HTML已设置");

    // 查询布局信息
    println!("\n=== 布局信息 ===");
    let rects = bridge.all_rects();
    println!("布局节点数量: {}", rects.len());
    
    if rects.is_empty() {
        println!("⚠ 警告: 没有布局节点！");
    }
    
    for (i, rect) in rects.iter().enumerate() {
        println!("\n节点 {}:", i);
        println!("  标签: {}", rect.tag_name);
        println!("  位置: ({}, {})", rect.x, rect.y);
        println!("  尺寸: {}x{}", rect.width, rect.height);
        println!("  背景: {:?}", rect.background);
    }

    // 渲染
    println!("\n=== 渲染 ===");
    let png_data = bridge.render();
    println!("PNG 数据大小: {} bytes", png_data.len());
    
    if png_data.len() < 100 {
        println!("⚠ 警告: PNG数据太小，可能渲染失败");
    }
    
    // 保存PNG到文件
    let output_path = "simple_test_output.png";
    match std::fs::write(output_path, &png_data) {
        Ok(_) => {
            println!("✓ PNG已保存到: {}", output_path);
            println!("请查看生成的图片文件");
        }
        Err(e) => {
            println!("✗ 保存PNG失败: {}", e);
        }
    }

    println!("\n=== 测试完成 ===");
}
