# AGENTS.md

This file establishes the context, architectural decisions, and development
guidelines for AI agents and developers working on the `appjs` repository.

## 1. Project Overview

`appjs` is a high-performance JavaScript/TypeScript Runtime designed for
building Desktop Applications. It leverages Rust for native capabilities and
runs JavaScript/TypeScript in a Bun subprocess.

### Core Philosophy

- **Native Performance**: Critical UI and system operations run in Rust.
- **Web Flexibility**: Application logic and UI definitions are driven by JS/TS.
- **Thread Safety**: Strict separation of concerns via a dual-threaded model.

### Architecture

The application implements a strict **Dual-Threaded Architecture**:

1. **Main Thread (UI Thread)**
   - **Responsibility**: Owns the `winit` Window and Event Loop. Manages the
     `masonry` Widget tree and rendering.
   - **Constraints**: NEVER perform blocking I/O or heavy computation here.
     Freezing this thread freezes the application window.
   - **Libraries**: `winit`, `masonry`, `masonry_winit`.

2. **Background Thread (JS Runtime Thread)**
   - **Responsibility**: hosts the Bun process bridge. Executes all JavaScript
     code, manages application state, and handles business logic.
   - **Constraints**: Cannot access UI objects directly. Must send messages to
     the Main Thread to mutate the UI.
   - **Libraries**: `std::process`, `rmp-serde`, and Bun in PATH.

3. **Communication Bridge**
   - Communication is asynchronous and message-based.
   - **Mechanism**: `std::sync::mpsc` for in-process UI event capture + MsgPack
     frames over Bun stdio for cross-process communication.
   - **Direction 1 (UI -> JS)**: User interactions (clicks, scroll, type) are
     captured by `winit`, converted to `UiEvent`s, encoded as MsgPack, and
     streamed to Bun.
   - **Direction 2 (JS -> UI)**: JS logic sends MsgPack command messages that
     are mapped to `JsCommand` and dispatched through `EventLoopProxy`.

## 2. Build, Test, and Run Commands

Standard Cargo workflows apply. Ensure you are in the project root.

### Build

Compile the project in debug mode:

```bash
cargo build
```

Compile for release (optimized):

```bash
cargo build --release
```

### Run

Run the application (Debug):

```bash
cargo run
```

_Note: This will launch the application window._

### Testing

Run the full test suite:

```bash
cargo test
```

**Run a Single Test**: To run a specific test case (e.g.,
`test_channel_communication`):

```bash
cargo test test_channel_communication -- --nocapture
```

- `--nocapture`: Displays `println!` output, essential for debugging async
  channel tests.

### Linting & Formatting

Ensure code quality before submitting changes.

**Linting**:

```bash
cargo clippy -- -D warnings
```

- Fix all warnings. Clippy captures idiomatic Rust issues that the compiler
  might miss.

**Formatting**:

```bash
cargo fmt
```

- Standard Rust formatting is mandatory.

## 3. Code Style & Guidelines

### Rust Conventions

- **Style**: Follow standard Rust naming conventions.
  - Types (`struct`, `enum`, `trait`): `PascalCase`
  - Functions, Methods, Variables, Modules: `snake_case`
  - Constants/Statics: `SCREAMING_SNAKE_CASE`
- **Imports**: Organize imports logically.
  ```rust
  // 1. Std lib
  use std::sync::mpsc;
  use std::thread;

  // 2. External crates
  use rmp_serde;
  use winit::event::Event;

  // 3. Local modules
  use crate::bridge::UiEvent;
  ```
- **Error Handling**:
  - Use `Result<T, E>` for recoverable errors.
  - Avoid `unwrap()` in production code. Use `expect("Reason")` if you are 100%
    sure, or better yet, handle the `Err` case.
  - Propagate errors using the `?` operator.

### Architectural Patterns

#### The Event Loop & Channels

When implementing features, always consider the flow of data across the thread
boundary.

**1. Defining Events** Create strong types for messages. Do not send raw
strings.

```rust
// src/events.rs (Example)
pub enum UiEvent {
    WindowResized { width: u32, height: u32 },
    MouseClick { x: f64, y: f64 },
}

pub enum JsCommand {
    SetTitle(String),
    CreateWidget { id: String, kind: String },
}
```

**2. The UI Loop (Main)** Inside the `winit` event loop:

- **Poll**: Check the receiver channel for `JsCommand` messages non-blocking
  (e.g., `try_recv`).
- **Dispatch**: Apply valid commands to the `masonry` widget tree.
- **Send**: Convert `winit` events to `UiEvent` and send to the JS thread.

**3. The JS Loop (Background)**

- Spawn Bun with `std::process::Command`.
- Forward `UiEvent`s to Bun via MsgPack frames over stdin.
- Read MsgPack command frames from Bun stdout and convert them to `JsCommand`.

### Bun Runtime Specifics

- Ensure `bun` is available in PATH.
- Keep protocol messages in `src/ipc/msgpack.rs` synchronized with
  `packages/appjs-runtime/src/bun_bridge.ts`.
- Use length-prefixed MsgPack frames for robust streaming over stdio.

## 4. Feature Implementation Checklist

When tasked with adding a new feature (e.g., "Add a button that logs to
console"):

1. **Plan the Message**: Add `ButtonClicked` to `UiEvent`.
2. **Update UI (Rust)**: Add the Button widget in `masonry`.
3. **Wire Event (Rust)**: In the UI thread, catch the button click and send
   `UiEvent::ButtonClicked`.
4. **Handle in JS (Rust/JS)**: Ensure the JS runtime receives this event and
   triggers a callback.
5. **Verify**: Run `cargo test` and `cargo run` to interact with the feature.

## 5. Environment & Tooling rules

- **Cursor/Copilot**:
  - When generating code, always prioritize type safety.
  - If you generate a `match` statement, ensure all arms are covered.
  - Do not hallucinate external crate features. Check `Cargo.toml` versions.
  - When editing `main.rs`, preserve the thread setup boilerplate unless
    explicitly asked to refactor the core architecture.

## 7. External Resources & Reference

- **Masonry Examples**: Masonry is part of the Xilem project. Search for usage
  patterns in the `linebender/xilem` repository, specifically under
  `masonry/examples`.
- **Bun Runtime**: Refer to `bun.sh/docs` for runtime behavior and Node-compat
  APIs.

## 8. Internal Documentation

For more detailed architectural overviews and refactoring logs, refer to the
`docs/` directory:

- **[Codebase Structure & Architecture](docs/architecture.md)**: A comprehensive
  guide to the folder structure, file responsibilities, and thread interactions.
