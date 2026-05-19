# Servo-Zero

A Web-Native browser engine project based on Servo, providing a bridge layer for DOM manipulation and event handling.

## Project Overview

This project implements a Web-Native bridge layer that enables:
- DOM injection and manipulation through the bridge
- Event listening and handling
- Network request management
- File operations

## Architecture

```
servo-zero/
├── bridge/              # Web-Native bridge implementation
│   ├── bridge.rs       # WebNativeBridge trait definition
│   ├── servo_impl.rs   # ServoBridge mock implementation
│   └── network.rs      # HTTP response types
├── components/         # Servo engine components
│   ├── shared/        # Shared dependencies (base, config, constellation, etc.)
│   ├── servo/         # Main servo component
│   ├── geometry/      # Geometry utilities
│   ├── layout/        # Layout engine
│   ├── paint/         # Rendering layer
│   └── script/        # Script engine
└── Cargo.toml         # Workspace configuration
```

## JavaScript Files in Project

Although this is a Web-Native project that does not use JavaScript, the following JS files are included as part of the Servo engine's media control functionality:

1. `components/shared/script/resources/media-controls.js` - HTML5 media player controls
2. `components/script/resources/media-controls.js` - HTML5 media player controls (duplicate)

These files are part of the Servo engine's built-in media control system for HTML5 `<video>` and `<audio>` elements.

## Building

```bash
# Build the bridge crate
cd bridge
cargo build

# Run tests
cargo test
```

## Usage Example

```rust
use servo_bridge::ServoBridge;
use servo_bridge::WebNativeBridge;

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

// Render
let png_data = bridge.render();
```

## Acknowledgments

This project was developed with assistance from **Trae**.

Special thanks to the Servo project team for providing the browser engine foundation.

## License

MPL-2.0