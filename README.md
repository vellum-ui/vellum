# Vellum UI

> **A JavaScript/TypeScript runtime for building native desktop applications.**
>
> Powered by Rust, Bun, and GPU-accelerated rendering.

> [!WARNING]
> **This project is in very early stages of development.** APIs are unstable and
> will change. It is not yet suitable for production use. Contributions,
> feedback, and ideas are very welcome.

---

## What is Vellum UI?

Vellum UI is a lightweight desktop application runtime that lets you build native,
GPU-rendered UIs using JavaScript or TypeScript. Instead of bundling a full web
browser (like Electron), Vellum UI pairs a Bun process with a native widget toolkit
([Masonry](https://github.com/linebender/xilem)), giving you:

- **Small binary size** -- no bundled Chromium
- **Native rendering** -- GPU-accelerated via
  [Vello](https://github.com/linebender/vello), not a web view
- **Fast startup** -- milliseconds, not seconds
- **Low memory footprint** -- native widgets, not a DOM

Think of it as: _what if Bun and a native UI toolkit had a baby?_

## Quick Start

### Prerequisites

- Rust toolchain (stable, 1.85+)
- A system with GPU support (Vulkan, Metal, or DX12)

### Build & Run

Ensure the application logic uses an up-to-date physical binary. **Always** compile the release/debug binary first before testing TS/JS interactions because running external scripts spawns the physical Rust executable.

```bash
# Clone the repo
git clone https://github.com/user/Vellum UI.git
cd Vellum UI

# Build the physical binary (mandatory before testing JS examples)
cargo build

# Run an example
bun run examples/solid/solid_counter.tsx
```

### Hello World

```tsx
// src/app.tsx
import * as Vellum from "@vellum/core";
import { createVellumRenderer, createSignal } from "@vellum/solid";

// 1. Configure OS Window
Vellum.window.setTitle("Hello World");
Vellum.body.setStyle({ background: "#1e1e2e", padding: 24, gap: 16 });

const renderer = createVellumRenderer(Vellum);

// 2. Build declarative components
function App() {
    const [count, setCount] = createSignal(0);

    return (
        <column gap={16}>
            <label text="Hello from Vellum UI!" fontSize={24} fontWeight="bold" color="#cdd6f4" />
            
            <button onClick={() => setCount(c => c + 1)}>
                <label text={() => `Clicked ${count()} times`} />
            </button>
        </column>
    );
}

// 3. Mount to screen
renderer.render(() => <App />);
```

```bash
bun run src/app.tsx
```

## Architecture

Vellum UI uses a strict **dual-threaded architecture** to keep the UI responsive at
all times:

```
+---------------------------+          +---------------------------+
|      Main Thread (UI)     |          |   Background Thread (IPC) |
|                           |          |                           |
|  winit Event Loop         |          |  External Client Process  |
|  Masonry Widget Tree      |  <---->  |  Application Logic        |
|  GPU Rendering (Vello)    |          |  State Management         |
|                           |          |                           |
+---------------------------+          +---------------------------+
        ^           |                        |           ^
        |           v                        v           |
   User Input    Render                 ClientCommand    UiEvent
  (pointer,     (Vello/                (create,     (click,
   keyboard)    wgpu)                  style,       resize,
                                       update)      value)
```

### Main Thread (UI Thread)

Owns the window and the widget tree. Handles all rendering and user input. This
thread **never blocks** -- no I/O, no heavy computation, no JS execution.

- **Window management**: `winit` event loop
- **Widget tree**: `masonry` (from the
  [Xilem](https://github.com/linebender/xilem) project)
- **Rendering**: Vello (GPU-accelerated 2D rendering)

### Background Thread (JS Runtime)

Runs JavaScript/TypeScript in a dedicated Bun subprocess. Executes all
application logic, manages state, and sends commands to the UI thread.

- **Runtime**: `bun` (external process)
- **Module system**: ES modules
- **IPC transport**: MsgPack over stdio (length-prefixed frames)

### Communication Bridge

The two threads communicate via asynchronous, typed message passing:

| Direction    | Mechanism          | Purpose                                                                            |
| ------------ | ------------------ | ---------------------------------------------------------------------------------- |
| **Client -> UI** | `EventLoopProxy`   | Widget creation, styling, updates. Zero-polling, immediately wakes the event loop. |
| **UI -> Client** | MsgPack over UDS Socket | User interactions (clicks, input, resize) streamed to the client process.             |

All messages are strongly typed Rust enums (`ClientCommand`, `UiEvent`) -- no raw
strings cross the thread boundary.

## API Overview

Vellum UI prioritizes declarative UI authoring using SolidJS + TSX. While an imperative `@vellum/core` API exists under the hood to bridge IPC, end-users should interact with the `@vellum/solid` bindings.

### Window & Body Configuration

Before mounting your declarative UI tree, you can configure the native OS window properties directly:

```typescript
import * as Vellum from "@vellum/core";

// Native Window APIs
Vellum.window.setTitle("My App");
Vellum.window.resize(1024, 768);
Vellum.window.close();

// Root Container Styling
Vellum.body.setStyle({
    background: "#1e1e2e",
    padding: 24,
});
```

### Declarative UI (SolidJS)

Initialize the custom renderer and mount your application:

```tsx
import { createVellumRenderer, createSignal } from "@vellum/solid";

const renderer = createVellumRenderer(Vellum);

function App() {
    const [name, setName] = createSignal("");

    return (
        <column gap={16}>
            <textInput 
                placeholder="Enter your name..." 
                onTextChanged={(e) => setName(e.value)} 
            />
            
            <label 
                text={() => `Hello, ${name() || "World"}!`} 
                fontSize={24} 
                color="#cdd6f4" 
            />
        </column>
    );
}

renderer.render(() => <App />);
```

### Styling Variables

Styles are passed directly as TSX attributes. Every component supports SolidJS reactive accessors (signals) seamlessly updating native padding, colors, sizing, and typography:

```tsx
<label 
    text="Dynamic styling"
    color={() => isActive() ? "#a6e3a1" : "#f38ba8"}
    fontSize={16}
    padding={{ top: 10, bottom: 10, left: 20, right: 20 }}
    background="#1e1e2e"
    cornerRadius={8}
/>
```

## Available Widgets (TSX)

All native Core Widgets are exposed natively as intrinsic JSX elements:

| Element | Description | Key Props |
|---------|-------------|-----------|
| `<label>` | Static text display | `text`, `fontSize`, `color`, `fontWeight` |
| `<button>` | Clickable button | `onClick` |
| `<checkbox>` | Toggle checkbox | `checked`, `onValueChanged` |
| `<textInput>` | Single-line text input | `placeholder`, `onTextChanged` |
| `<slider>` | Range slider | `min`, `max`, `value`, `onValueChanged` |
| `<progressBar>` | Progress indicator | `progress` (0.0 - 1.0) |
| `<spinner>` | Loading indicator | |
| `<prose>` | Selectable read-only text | `text`, CSS text styles |
| `<svg>` | Vector icons/graphics | `svg_data` (raw SVG string) |
| `<image>` | Bitmap image display | `data` (Uint8Array), `objectFit` |
| `<column>` | Vertical flex layout | `gap`, `crossAxisAlignment`, `mainAxisAlignment` |
| `<row>` | Horizontal flex layout | `gap`, `crossAxisAlignment`, `mainAxisAlignment` |
| `<flex>` | Base flexbox layout | `direction`, `gap`, `flex` |
| `<box>` | Fixed-size container (SizedBox) | `width`, `height` |
| `<zstack>` | Z-Index overlapping stack | |
| `<portal>` | Scrollable view port | |

## Examples

See the [`examples/`](examples/) directory:

- **[`test_ui.js`](examples/test_ui.js)** -- Widget gallery showcasing every
  widget type, styling, and event handling
- **[`styled_counter.js`](examples/styled_counter.js)** -- Counter app with
  dynamic styling
- **[`counter.js`](examples/counter.js)** -- Minimal counter example
- **[`solid_counter.ts`](examples/solid_counter.ts)** -- SolidJS-powered counter
  using `@vellum/solid`
- **[`solid_counter.tsx`](examples/solid_counter.tsx)** -- Solid TSX example
  rendered through `@vellum/solid` (declarative accessor props for
  `text`, `style`, and dynamic label state)

## Roadmap

This project is in its early stages. Here's what's planned:

- [ ] **TypeScript support** -- Run `.ts` files directly with type checking
- [ ] **Hot Module Replacement (HMR)** -- Live-reload UI changes during
      development
- [ ] **Rust extension API** -- Write native Rust plugins that expose new
      capabilities to JS (custom widgets, system APIs, hardware access)
- [ ] **Remote script loading** -- Run scripts from HTTPS URLs, npm, and JSR
      registries
- [ ] **Permission system** -- Fine-grained permissions (file system, network,
      env) with secure defaults, similar to Deno's model
- [ ] **Strict sandboxing** -- Apps run in a sandbox by default with no access
      to the system unless explicitly granted
- [ ] **Shebang support** -- Add `#!/usr/bin/env Vellum UI` to scripts and run them
      directly as executables
- [ ] **More widgets** -- Tables, trees, menus, dialogs, tabs, images
- [ ] **Multi-window support** -- Open and manage multiple windows from a single
      script
- [ ] **Platform integration** -- System tray, notifications, file dialogs,
      clipboard
- [ ] **Accessibility** -- Full keyboard navigation and screen reader support
      (partially supported via Masonry)

## Tech Stack

| Component    | Technology                                       | Purpose                            |
| ------------ | ------------------------------------------------ | ---------------------------------- |
| JS Engine    | [Bun](https://bun.sh/)                           | JavaScript/TypeScript execution    |
| UI Framework | [Masonry](https://github.com/linebender/xilem)   | Native widget tree                 |
| Windowing    | [winit](https://github.com/rust-windowing/winit) | Cross-platform window management   |
| 2D Rendering | [Vello](https://github.com/linebender/vello)     | GPU-accelerated vector graphics    |
| Text Layout  | [Parley](https://github.com/linebender/parley)   | Text shaping and layout            |
| Language     | Rust                                             | Performance, safety, native access |

## Contributing

This is an early-stage project and contributions are very welcome! Whether it's:

- Bug reports and feature requests
- Code contributions (Rust or JS/TS)
- Documentation improvements
- Name suggestions (seriously, `Vellum UI` is a placeholder!)

Please open an issue to discuss before submitting large PRs.

## License

TBD

---

_Built with Rust, Bun, and the Linebender ecosystem._
