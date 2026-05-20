//! 测试渲染流程

use servo_bridge::{RealServoBridge, WebNativeBridge};

fn main() {
    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    println!("=== 测试渲染流程 ===\n");

    // 创建桥接器
    let mut bridge = RealServoBridge::new(800, 600);
    println!("✓ 创建桥接器: 800x600");

    // 设置简单的HTML内容
    let html = r#"
        <div id="app" style="background-color: blue; width: 200px; height: 100px;">
            <div id="child" style="background-color: red; width: 100px; height: 50px;">
                Child
            </div>
        </div>
    "#;

    println!("\n设置HTML内容...");
    bridge.set_html(html);
    println!("✓ HTML已设置");
    
    // 调试：检查DOM树
    println!("\n=== DOM树信息 ===");
    // 无法直接访问，但可以通过布局节点间接检查

    // 查询布局信息
    println!("\n=== 布局信息 ===");
    let rects = bridge.all_rects();
    println!("布局节点数量: {}", rects.len());
    
    if rects.is_empty() {
        println!("⚠ 警告: 没有布局节点！");
        println!("可能原因:");
        println!("1. HTML解析失败");
        println!("2. 布局计算失败");
        println!("3. 元素display属性为none");
    }
    
    for (i, rect) in rects.iter().enumerate() {
        println!("\n节点 {}:", i);
        println!("  标签: {}", rect.tag_name);
        println!("  位置: ({:.1}, {:.1})", rect.x, rect.y);
        println!("  尺寸: {:.1}x{:.1}", rect.width, rect.height);
        if let Some(color) = &rect.background {
            println!("  背景色: RGB({}, {}, {}, alpha={})", color.r, color.g, color.b, color.a);
        } else {
            println!("  背景色: 无 (将显示为浅灰色)");
        }
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
