# Servo-Zero

A lightweight Web-Native browser rendering engine based on Servo, providing a pure Rust bridge layer for DOM manipulation, CSS layout, and event handling. **No SpiderMonkey dependency.**

## Project Overview

Servo-Zero implements a Web-Native bridge layer that enables:
- DOM injection and manipulation through the bridge
- CSS parsing, cascade, and selector matching
- CSS box model and Flexbox layout engine
- Event listening and handling (click, form submit, window.open)
- PNG image rendering using tiny-skia
- HTTP networking support (optional)
- File operations

## Architecture

```
servo-zero/
├── Cargo.toml              # Workspace configuration
├── bridge/                 # Web-Native bridge implementation
│   ├── lib.rs            # Library entry point & exports
│   ├── bridge.rs         # WebNativeBridge trait definition
│   ├── servo_impl.rs     # Mock ServoBridge implementation (for testing)
│   ├── real_impl.rs      # Production RealServoBridge implementation
│   └── network.rs        # HTTP response types
└── components/            # Core engine components
    ├── html_core/        # HTML parsing & DOM (pure Rust)
    ├── css_core/        # CSS parsing, cascade & matching
    └── layout_core/     # CSS box model & Flexbox layout (using taffy)
```

## Features

### Triple Mode Support

| Mode | Features | Use Case |
|------|----------|----------|
| **Default** (`network`) | HTML, CSS, Layout, Rendering, Network | Full browser engine |
| **Embed** (`network`) | Same as default, optimized for embedding | Embedded UI rendering |
| **Minimal** (no features) | CSS, Layout, Rendering only | Minimal binary size |

### Core Components

- **html_core**: Pure Rust HTML5 parser with DOM tree support
- **css_core**: CSS parser with selector matching, cascade, and color/length parsing
- **layout_core**: CSS box model, Flexbox layout algorithm (via taffy), and pixel rendering
- **bridge**: WebNativeBridge trait with dual implementations (mock + production)

### Network Support

Optional HTTP networking via `reqwest`:
- GET/POST requests
- URL navigation
- File downloads
- Response handling with headers and status codes

## Building

```bash
# Build with default features (network enabled)
cargo build

# Build with all features
cargo build --features "network"

# Build in minimal mode (no network, smaller binary)
cargo build --no-default-features

# Run tests
cargo test
```

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `network` | On | Enable HTTP networking via reqwest |
| `embed` | Off | Embed mode (currently same as network) |

## Usage Example

```rust
use servo_bridge::{RealServoBridge, WebNativeBridge};

// Create bridge
let mut bridge = RealServoBridge::new(1280, 720);

// Set HTML content
bridge.set_html(r#"
    <html>
        <head>
            <style>
                #app { background: blue; width: 100px; height: 100px; }
            </style>
        </head>
        <body>
            <div id="app">
                <button id="btn">Click</button>
            </div>
        </body>
    </html>
"#);

// Query elements
let btn_id = bridge.query("#btn");

// Bind click events
bridge.on_click("#btn", Box::new(|x, y| {
    println!("Button clicked at ({}, {})", x, y);
}));

// Handle clicks
bridge.handle_click(100.0, 200.0);

// Render to PNG
let png_data = bridge.render();
```

### Network Example

```rust
// Navigate to URL
bridge.navigate("https://example.com")?;

// HTTP GET
let response = bridge.http_get("https://api.example.com/data")?;
println!("Status: {}", response.status);
println!("Body: {}", response.text());

// Download file
let bytes = bridge.download_file("https://example.com/file.pdf", "/tmp/file.pdf")?;
println!("Downloaded {} bytes", bytes);
```

## Dependencies

### Runtime
- `tiny-skia` - 2D rendering
- `png` - PNG encoding
- `log` - Logging
- `serde` / `serde_json` - Serialization

### Core Components
- `html-core` - HTML parsing and DOM (internal)
- `css-core` - CSS parsing and cascade (internal)
- `layout-core` - Layout engine with taffy (internal)
- `euclid` - Geometry types
- `taffy` - Flexbox layout algorithm
- `rayon` - Parallel processing
- `rustc-hash` - Fast hashing

### Optional
- `reqwest` - HTTP networking (with `network` feature)
- `url` - URL parsing

## Project Structure

### Bridge Layer

The bridge provides a unified `WebNativeBridge` trait with two implementations:

1. **ServoBridge** (mock): For testing and reference
2. **RealServoBridge** (production): Full implementation with HTML parsing, CSS, layout, and rendering

### Core Components

Each component is a separate crate for modularity:

- **html_core**: Parses HTML into DOM trees
- **css_core**: Parses CSS, handles cascade, resolves styles
- **layout_core**: Builds layout trees, computes positions using Flexbox

## Acknowledgments

This project was developed with assistance from **CodeBuddy** and **Trae**.

Special thanks to the Servo project team for providing the browser engine foundation.

## License

MPL-2.0
