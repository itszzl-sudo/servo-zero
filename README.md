# Servo-Zero

A lightweight Web-Native browser rendering engine based on Servo, providing a pure Rust bridge layer for DOM manipulation, CSS layout, and event handling. **No SpiderMonkey dependency.**

## Project Overview

Servo-Zero implements a Web-Native bridge layer that enables:
- DOM injection and manipulation through the bridge
- CSS box model and Flexbox layout engine
- Event listening and handling
- PNG image rendering

## Architecture

```
servo-zero/
├── Cargo.toml              # Workspace configuration
├── bridge/                 # Web-Native bridge implementation
│   ├── lib.rs            # Library entry point & exports
│   ├── bridge.rs         # WebNativeBridge trait definition
│   ├── servo_impl.rs     # ServoBridge implementation
│   ├── real_impl.rs      # Production bridge implementation
│   └── network.rs        # HTTP response types
└── components/            # Core engine components
    ├── html_core/        # HTML parsing & DOM
    ├── css_core/        # CSS parsing, cascade & matching
    └── layout_core/     # CSS box model & Flexbox layout
```

## Features

### Dual Mode Support

| Mode | Features | Use Case |
|------|----------|----------|
| **Full** (`html`) | HTML parsing, DOM, CSS, Layout, Rendering | Full browser engine |
| **Embed** | CSS, Layout, Rendering (no HTML) | Embedded UI rendering |

### Core Components

- **html_core**: Pure Rust HTML5 parser with DOM tree support
- **css_core**: CSS 2.1/3 parser with selector matching and cascade
- **layout_core**: CSS box model, Flexbox layout algorithm, and pixel rendering

## Building

```bash
# Build with HTML support (full mode)
cargo build --features html

# Build in embed mode (no HTML, smaller binary)
cargo build --no-default-features --features embed

# Build default configuration
cargo build

# Run tests
cargo test
```

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `html` | Off | Enable HTML parsing and DOM support |
| `embed` | Off | Embed mode: no HTML, minimal binary size |

## Usage Example

```rust
use servo_bridge::ServoBridge;
use servo_bridge::WebNativeBridge;

// Create bridge
let mut bridge = ServoBridge::new(1280, 720);

// Set HTML content
bridge.set_html(r#"<div id="app"><button id="btn">Click</button></div>"#);

// Query elements
let btn_id = bridge.query("#btn").unwrap();

// Bind click events
bridge.on_click("#btn", Box::new(|x, y| {
    println!("Button clicked at ({}, {})", x, y);
}));

// Handle clicks
bridge.handle_click(100.0, 200.0);

// Render to PNG
let png_data = bridge.render();
```

## Dependencies

### Runtime
- `tiny-skia` - 2D rendering
- `png` - PNG encoding
- `log` - Logging
- `serde` - Serialization

### Core (embedded)
- `html5ever` - HTML parsing (optional, with `html` feature)
- `cssparser` - CSS parsing (optional, with `html` feature)
- `selectors` - CSS selector matching (optional, with `html` feature)

## Acknowledgments

This project was developed with assistance from **CodeBuddy** and **Trae**.

Special thanks to the Servo project team for providing the browser engine foundation.

## License

MPL-2.0
