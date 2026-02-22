// AppJS - JavaScript Desktop Runtime
//
// This application implements a dual-threaded architecture:
// - Main Thread (UI): Owns the window and widget tree via masonry_winit
// - Background Thread (JS): Runs a Bun subprocess bridge
//
// Communication between threads uses EventLoopProxy (JS→UI, zero polling)
// and MsgPack over Bun stdio (UI→JS, for UI events).

// On Windows platform, don't show a console when opening the app.
// #![windows_subsystem = "windows"]

mod ipc;
mod socket;
mod ui;

use std::thread;

use ipc::IpcChannels;
use ipc::server::run_ipc_server;
use ui::{prepare_ui, run_ui_blocking};

fn main() {
    println!("AppJS Starting...");

    let rust_log = std::env::var("RUST_LOG").ok();
    let should_override_log = match rust_log.as_deref() {
        Some(value) => value.contains("debug") || value.contains("trace"),
        None => true,
    };
    if should_override_log {
        unsafe {
            std::env::set_var("RUST_LOG", "warn");
        }
        println!("[Main] RUST_LOG set to info");
    }

    #[cfg(target_os = "windows")]
    if std::env::var_os("WGPU_BACKEND").is_none() {
        unsafe {
            std::env::set_var("WGPU_BACKEND", "gl");
        }
        println!("[Main] WGPU backend defaulted to OpenGL (WGPU_BACKEND=gl)");
    }

    println!("[Main] Operating in Client-Server Socket IPC Mode");

    // Phase 1: Build the EventLoop and extract EventLoopProxy (non-blocking).
    // This must happen before spawning the JS thread so the proxy can be shared.
    let (ui_setup, event_loop) = prepare_ui();

    // Phase 2: Create IPC channels with the EventLoopProxy.
    // JS→UI commands use EventLoopProxy (immediately wakes the event loop, zero polling).
    // UI→JS events use mpsc channels.
    let channels = IpcChannels::new(ui_setup.proxy, ui_setup.window_id);

    let ui_channels = channels.ui;
    let js_channels = channels.ipc_server;

    // Phase 3: Spawn the IPC server thread with EventLoopProxy-based command sender.
    let ipc_server_handle = thread::Builder::new()
        .name("ipc-server".to_string())
        .spawn(move || {
            println!("[Main] IPC server thread started");
            run_ipc_server(js_channels);
            println!("[Main] IPC server thread finished");
        })
        .unwrap_or_else(|e| panic!("Fatal: failed to spawn IPC server thread: {e}"));

    // Phase 4: Run the UI event loop on the main thread (blocks forever).
    // The main thread MUST run the UI due to platform requirements (macOS, etc.).
    println!("[Main] Starting UI on main thread");
    run_ui_blocking(event_loop, ui_setup.window_id, ui_channels.event_sender);

    // Wait for the IPC server thread to finish after the UI closes
    println!("[Main] UI closed, waiting for IPC server thread to finish...");
    if let Err(e) = ipc_server_handle.join() {
        eprintln!("[Main] IPC server thread panicked: {:?}", e);
    }

    println!("[Main] AppJS shutdown complete");
}
