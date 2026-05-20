use servo_bridge::{RealServoBridge, WebNativeBridge};

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    println!("=== 高级 CSS 特性测试 ===\n");
    
    test_flexbox();
    test_text_styles();
    test_borders();
    test_gradients();
    
    println!("\n=== 所有测试完成 ===");
}

fn test_flexbox() {
    println!("\n--- 测试 Flexbox 布局 ---");
    let mut bridge = RealServoBridge::new(800, 200);
    
    let html = r#"
        <div style="display: flex; width: 800px; height: 200px; background: #f0f0f0;">
            <div style="flex: 1; background: #ff6b6b;">Item 1</div>
            <div style="flex: 2; background: #4ecdc4;">Item 2</div>
            <div style="flex: 1; background: #45b7d1;">Item 3</div>
        </div>
    "#;
    
    bridge.set_html(html);
    let png = bridge.render();
    std::fs::write("test_flexbox.png", &png).unwrap();
    println!("✓ Flexbox 渲染: test_flexbox.png ({} bytes)", png.len());
}

fn test_text_styles() {
    println!("\n--- 测试文本样式 ---");
    let mut bridge = RealServoBridge::new(600, 300);
    
    let html = r#"
        <div style="width: 600px; height: 300px; background: white; padding: 20px;">
            <h1 style="font-size: 32px; color: #333;">标题文本</h1>
            <p style="font-size: 16px; color: #666;">这是一段普通文本。</p>
            <p style="font-size: 14px; color: #999; font-style: italic;">斜体文本</p>
        </div>
    "#;
    
    bridge.set_html(html);
    let png = bridge.render();
    std::fs::write("test_text_styles.png", &png).unwrap();
    println!("✓ 文本样式渲染: test_text_styles.png ({} bytes)", png.len());
}

fn test_borders() {
    println!("\n--- 测试边框 ---");
    let mut bridge = RealServoBridge::new(400, 200);
    
    let html = r#"
        <div style="width: 400px; height: 200px; background: white; padding: 20px;">
            <div style="width: 100px; height: 100px; border: 2px solid red; background: #ffe0e0;"></div>
            <div style="width: 100px; height: 100px; border-radius: 10px; background: #e0ffe0;"></div>
        </div>
    "#;
    
    bridge.set_html(html);
    let png = bridge.render();
    std::fs::write("test_borders.png", &png).unwrap();
    println!("✓ 边框渲染: test_borders.png ({} bytes)", png.len());
}

fn test_gradients() {
    println!("\n--- 测试渐变 ---");
    let mut bridge = RealServoBridge::new(600, 150);
    
    let html = r#"
        <div style="width: 600px; height: 150px;">
            <div style="width: 300px; height: 150px; background: linear-gradient(to right, #ff6b6b, #4ecdc4);"></div>
            <div style="width: 300px; height: 150px; background: linear-gradient(to bottom, #45b7d1, #f9ca24);"></div>
        </div>
    "#;
    
    bridge.set_html(html);
    let png = bridge.render();
    std::fs::write("test_gradients.png", &png).unwrap();
    println!("✓ 渐变渲染: test_gradients.png ({} bytes)", png.len());
}
