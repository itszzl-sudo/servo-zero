use servo_bridge::{RealServoBridge, WebNativeBridge};

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    println!("=== 字体渲染诊断测试 ===\n");
    
    let mut bridge = RealServoBridge::new(400, 200);
    
    let html = r#"
        <div style="width: 400px; height: 200px; background: white; padding: 20px;">
            <h1 style="font-size: 24px; color: black; background: #eee;">Hello World</h1>
            <p style="font-size: 16px; color: black; background: #ddd;">测试中文文本</p>
        </div>
    "#;
    
    bridge.set_html(html);
    
    let rects = bridge.all_rects();
    println!("布局节点:");
    for (i, rect) in rects.iter().enumerate() {
        println!("  {}: {} at ({:.1}, {:.1}) size {:.1}x{:.1}", 
            i, rect.tag_name, rect.x, rect.y, rect.width, rect.height);
    }
    
    let png = bridge.render();
    std::fs::write("font-test.png", &png).unwrap();
    println!("\n✓ 已保存: font-test.png ({} bytes)", png.len());
}
