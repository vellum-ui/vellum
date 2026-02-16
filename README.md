# appjs

> **A JavaScript/TypeScript runtime for building native desktop applications.**
>
> Powered by Rust, V8, and GPU-accelerated rendering.

> [!WARNING]
> **This project is in very early stages of development.** APIs are unstable and will change. It is not yet suitable for production use. Contributions, feedback, and ideas are very welcome.

> [!NOTE]
> **The name `appjs` is a placeholder** and we're open to suggestions for a better name. If you have ideas, feel free to open an issue!

---

## What is appjs?

appjs is a lightweight desktop application runtime that lets you build native, GPU-rendered UIs using JavaScript or TypeScript. Instead of bundling a full web browser (like Electron), appjs pairs a minimal V8-based JS runtime ([Deno Core](https://github.com/denoland/deno_core)) with a native widget toolkit ([Masonry](https://github.com/linebender/xilem)), giving you:

- **Small binary size** -- no bundled Chromium
- **Native rendering** -- GPU-accelerated via [Vello](https://github.com/linebender/vello), not a web view
- **Fast startup** -- milliseconds, not seconds
- **Low memory footprint** -- native widgets, not a DOM

Think of it as: *what if Deno and a native UI toolkit had a baby?*

## Quick Start

### Prerequisites

- Rust toolchain (stable, 1.85+)
- A system with GPU support (Vulkan, Metal, or DX12)

### Build & Run

```bash
# Clone the repo
git clone https://github.com/user/appjs.git
cd appjs

# Build
cargo build

# Run an example
cargo run -- examples/test_ui.js
```

### Hello World

```javascript
// hello.js
appjs.window.setTitle("Hello World");

appjs.body.setStyle({
    background: "#1e1e2e",
    padding: 24,
    gap: 16,
});

appjs.label("greeting", null, "Hello from appjs!", {
    fontSize: 24,
    fontWeight: "bold",
    color: "#cdd6f4",
});

appjs.button("btn", null, "Click me!");

appjs.events.on("widgetAction", (e) => {
    if (e.widgetId === "btn" && e.action === "click") {
        appjs.ui.setText("greeting", "You clicked the button!");
    }
});
```

```bash
cargo run -- hello.js
```

## Architecture

appjs uses a strict **dual-threaded architecture** to keep the UI responsive at all times:

```
+---------------------------+          +---------------------------+
|      Main Thread (UI)     |          |   Background Thread (JS)  |
|                           |          |                           |
|  winit Event Loop         |          |  Deno Core (V8)           |
|  Masonry Widget Tree      |  <---->  |  Application Logic        |
|  GPU Rendering (Vello)    |          |  State Management         |
|                           |          |                           |
+---------------------------+          +---------------------------+
        ^           |                        |           ^
        |           v                        v           |
   User Input    Render                 JsCommand    UiEvent
  (pointer,     (Vello/                (create,     (click,
   keyboard)    wgpu)                  style,       resize,
                                       update)      value)
```

### Main Thread (UI Thread)

Owns the window and the widget tree. Handles all rendering and user input. This thread **never blocks** -- no I/O, no heavy computation, no JS execution.

- **Window management**: `winit` event loop
- **Widget tree**: `masonry` (from the [Xilem](https://github.com/linebender/xilem) project)
- **Rendering**: Vello (GPU-accelerated 2D rendering)

### Background Thread (JS Runtime)

Runs the V8 JavaScript engine via `deno_core`. Executes all application logic, manages state, and sends commands to the UI thread.

- **Runtime**: `deno_core` with custom ops
- **Module system**: ES modules
- **Async**: Full async/await support

### Communication Bridge

The two threads communicate via asynchronous, typed message passing:

| Direction | Mechanism | Purpose |
|-----------|-----------|---------|
| **JS -> UI** | `EventLoopProxy` | Widget creation, styling, updates. Zero-polling, immediately wakes the event loop. |
| **UI -> JS** | `mpsc` channel | User interactions (clicks, input, resize). Delivered via async `op_wait_for_event`. |

All messages are strongly typed Rust enums (`JsCommand`, `UiEvent`) -- no raw strings cross the thread boundary.

## API Overview

### Window

```javascript
appjs.window.setTitle("My App");
appjs.window.resize(1024, 768);
appjs.window.close();
```

### Body (Root Container)

Style the root container like you would `<body>` in HTML:

```javascript
appjs.body.setStyle({
    background: "#1e1e2e",
    padding: 24,
    gap: 16,
    crossAxisAlignment: "fill",
});
```

### Widgets

All widgets follow the pattern: `appjs.widget(id, parentId, ...args, style?)`.

```javascript
// Text
appjs.label("id", parentId, "text", { fontSize: 16, color: "#fff" });
appjs.prose("id", parentId, "selectable text");

// Controls
appjs.button("id", parentId, "Click me");
appjs.checkbox("id", parentId, false, "Enable feature");
appjs.slider("id", parentId, 0, 100, 50);
appjs.textInput("id", parentId, "placeholder...");

// Layout
appjs.row("id", parentId, { gap: 8 });
appjs.column("id", parentId, { gap: 8, crossAxisAlignment: "fill" });
appjs.flex("id", parentId, { direction: "row", gap: 12 });
appjs.box("id", parentId, { width: 100, height: 100 });
appjs.zstack("id", parentId);
appjs.portal("id", parentId);  // scrollable container

// Feedback
appjs.progressBar("id", parentId, 0.5);
appjs.spinner("id", parentId);
```

### Styling

Every widget accepts a style object with CSS-like properties:

```javascript
// Text styles
{
    fontSize: 16,
    fontWeight: "bold",       // 100-900 or "normal", "bold"
    fontStyle: "italic",
    fontFamily: "monospace",
    color: "#cdd6f4",
    letterSpacing: 1.5,
    lineHeight: 1.4,
    textAlign: "center",     // "start", "center", "end", "justify"
    underline: true,
    strikethrough: true,
}

// Box styles
{
    background: "#1e1e2e",
    borderColor: "#585b70",
    borderWidth: 2,
    cornerRadius: 8,
    padding: 16,             // or [top, right, bottom, left]
    width: 200,
    height: 100,
}

// Flex layout styles
{
    direction: "row",        // "row" or "column"
    gap: 12,
    flex: 1,                 // flex grow factor
    crossAxisAlignment: "center",   // "start", "center", "end", "fill", "baseline"
    mainAxisAlignment: "spaceBetween", // "start", "center", "end", "spaceBetween", "spaceAround", "spaceEvenly"
    mustFillMainAxis: true,
}
```

Update styles at runtime:

```javascript
appjs.ui.setStyle("myWidget", { color: "#a6e3a1", fontSize: 20 });
appjs.ui.setStyleProperty("myWidget", "color", "#f38ba8");
```

### Events

```javascript
appjs.events.on("widgetAction", (e) => {
    // e.widgetId  -- which widget
    // e.action    -- "click", "valueChanged", "textChanged"
    // e.value     -- associated value (number, string)
});

appjs.events.on("windowResized", (e) => {
    console.log(e.width, e.height);
});

// Also: mouseClick, keyPress, keyRelease, windowFocusChanged, windowCloseRequested
```

### Widget Updates

```javascript
appjs.ui.setText("label1", "New text");
appjs.ui.setValue("slider1", 75);
appjs.ui.setChecked("checkbox1", true);
appjs.ui.setVisible("widget1", false);
appjs.ui.removeWidget("widget1");
```

## Available Widgets

| Widget | Description | Key Properties |
|--------|-------------|----------------|
| `label` | Static text display | `fontSize`, `color`, `fontWeight` |
| `button` | Clickable button | Emits `click` action |
| `checkbox` | Toggle with label | `checked`, emits `valueChanged` |
| `textInput` | Single-line text field | `placeholder` |
| `slider` | Range input | `min`, `max`, `value`, emits `valueChanged` |
| `progressBar` | Progress indicator | `progress` (0.0 - 1.0) |
| `spinner` | Loading indicator | Animated |
| `prose` | Selectable read-only text | Same text styles as label |
| `flex` / `row` / `column` | Flexbox layout | `direction`, `gap`, `crossAxisAlignment` |
| `box` (SizedBox) | Fixed-size container | `width`, `height` |
| `zstack` | Overlay/stack container | Children overlap |
| `portal` | Scrollable container | Wraps content |

## Examples

See the [`examples/`](examples/) directory:

- **[`test_ui.js`](examples/test_ui.js)** -- Widget gallery showcasing every widget type, styling, and event handling
- **[`styled_counter.js`](examples/styled_counter.js)** -- Counter app with dynamic styling
- **[`counter.js`](examples/counter.js)** -- Minimal counter example

## Roadmap

This project is in its early stages. Here's what's planned:

- [ ] **TypeScript support** -- Run `.ts` files directly with type checking
- [ ] **Hot Module Replacement (HMR)** -- Live-reload UI changes during development
- [ ] **Rust extension API** -- Write native Rust plugins that expose new capabilities to JS (custom widgets, system APIs, hardware access)
- [ ] **Remote script loading** -- Run scripts from HTTPS URLs, npm, and JSR registries
- [ ] **Permission system** -- Fine-grained permissions (file system, network, env) with secure defaults, similar to Deno's model
- [ ] **Strict sandboxing** -- Apps run in a sandbox by default with no access to the system unless explicitly granted
- [ ] **Shebang support** -- Add `#!/usr/bin/env appjs` to scripts and run them directly as executables
- [ ] **More widgets** -- Tables, trees, menus, dialogs, tabs, images
- [ ] **Multi-window support** -- Open and manage multiple windows from a single script
- [ ] **Platform integration** -- System tray, notifications, file dialogs, clipboard
- [ ] **Accessibility** -- Full keyboard navigation and screen reader support (partially supported via Masonry)

## Tech Stack

| Component | Technology | Purpose |
|-----------|-----------|---------|
| JS Engine | [Deno Core](https://github.com/denoland/deno_core) (V8) | JavaScript/TypeScript execution |
| UI Framework | [Masonry](https://github.com/linebender/xilem) | Native widget tree |
| Windowing | [winit](https://github.com/rust-windowing/winit) | Cross-platform window management |
| 2D Rendering | [Vello](https://github.com/linebender/vello) | GPU-accelerated vector graphics |
| Text Layout | [Parley](https://github.com/linebender/parley) | Text shaping and layout |
| Language | Rust | Performance, safety, native access |

## Contributing

This is an early-stage project and contributions are very welcome! Whether it's:

- Bug reports and feature requests
- Code contributions (Rust or JS/TS)
- Documentation improvements
- Name suggestions (seriously, `appjs` is a placeholder!)

Please open an issue to discuss before submitting large PRs.

## License

TBD

---

*Built with Rust, V8, and the Linebender ecosystem.*
