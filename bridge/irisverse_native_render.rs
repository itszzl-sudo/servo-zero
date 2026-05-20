use servo_bridge::{RealServoBridge, WebNativeBridge};
use std::fs;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    println!("=== Irisverse Native 渲染 ===\n");
    
    let html_path = "C:/Users/a/Documents/comate/irisverse-org/index-bundle.html";
    let output_path = "irisverse-native-render.png";
    
    println!("1. 读取 HTML 文件...");
    let html = match fs::read_to_string(html_path) {
        Ok(content) => {
            println!("   ✓ 读取成功: {} bytes", content.len());
            content
        }
        Err(e) => {
            eprintln!("   ✗ 读取失败: {}", e);
            return;
        }
    };
    
    println!("\n2. 创建桥接器 (1920x1080)...");
    let mut bridge = RealServoBridge::new(1920, 1080);
    println!("   ✓ 桥接器已创建");
    
    println!("\n3. 解析 HTML + CSS...");
    bridge.set_html(&html);
    println!("   ✓ HTML 已解析");
    
    println!("\n4. 查询布局信息...");
    let rects = bridge.all_rects();
    println!("   布局节点数量: {}", rects.len());
    
    for (i, rect) in rects.iter().take(10).enumerate() {
        println!("   - 节点 {}: {} ({:.1}x{:.1})", 
            i, rect.tag_name, rect.width, rect.height);
    }
    
    println!("\n5. 渲染到 PNG...");
    let png_data = bridge.render();
    println!("   ✓ PNG 大小: {} bytes", png_data.len());
    
    println!("\n6. 保存到文件...");
    match fs::write(output_path, &png_data) {
        Ok(_) => println!("   ✓ 已保存: {}", output_path),
        Err(e) => eprintln!("   ✗ 保存失败: {}", e),
    }
    
    println!("\n=== 渲染完成 ===");
}
