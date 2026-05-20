//! 测试渲染流程

use servo_bridge::{RealServoBridge, WebNativeBridge};

fn main() {
    println!("=== 测试渲染流程 ===\n");

    // 创建桥接器
    let mut bridge = RealServoBridge::new(800, 600);
    println!("✓ 创建桥接器: 800x600");

    // 设置简单的HTML内容
    let html = r#"
        <html>
            <head>
                <style>
                    body { background: red; }
                    #app { background: blue; width: 200px; height: 100px; }
                </style>
            </head>
            <body>
                <div id="app">
                    <button id="btn">Click Me</button>
                </div>
            </body>
        </html>
    "#;

    println!("\n设置HTML内容...");
    bridge.set_html(html);
    println!("✓ HTML已设置");

    // 查询布局信息
    println!("\n=== 布局信息 ===");
    let rects = bridge.all_rects();
    println!("布局节点数量: {}", rects.len());
    
    for (i, rect) in rects.iter().enumerate() {
        println!("\n节点 {}:", i);
        println!("  标签: {}", rect.tag_name);
        println!("  位置: ({}, {})", rect.x, rect.y);
        println!("  尺寸: {}x{}", rect.width, rect.height);
        println!("  背景: {:?}", rect.background);
    }

    // 测试特定选择器
    println!("\n=== 选择器查询 ===");
    if let Some(app_rect) = bridge.get_rect("#app") {
        println!("#app 矩形: ({}, {}, {}, {})", 
            app_rect.x, app_rect.y, app_rect.width, app_rect.height);
    } else {
        println!("✗ 未找到 #app");
    }

    if let Some(body_rect) = bridge.get_rect("body") {
        println!("body 矩形: ({}, {}, {}, {})", 
            body_rect.x, body_rect.y, body_rect.width, body_rect.height);
    } else {
        println!("✗ 未找到 body");
    }

    // 渲染
    println!("\n=== 渲染 ===");
    let png_data = bridge.render();
    println!("PNG 数据大小: {} bytes", png_data.len());
    
    // 保存PNG到文件
    let output_path = "test_output.png";
    if let Ok(_) = std::fs::write(output_path, &png_data) {
        println!("✓ PNG已保存到: {}", output_path);
    } else {
        println!("✗ 保存PNG失败");
    }

    println!("\n=== 诊断完成 ===");
}
