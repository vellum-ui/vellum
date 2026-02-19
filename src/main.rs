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
mod js_thread;
mod ui_thread;

use std::thread;

use ipc::IpcChannels;
use js_thread::{JsRuntimeConfig, run_js_thread};
use ui_thread::{prepare_ui, run_ui_blocking};

fn normalize_script_path_for_bun(path: &std::path::Path) -> String {
    let raw = path.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    {
        if let Some(stripped) = raw.strip_prefix(r"\\?\UNC\") {
            return format!(r"\\{}", stripped);
        }
        if let Some(stripped) = raw.strip_prefix(r"\\?\") {
            return stripped.to_string();
        }
    }

    raw
}

fn main() {
    println!("AppJS Starting...");

    #[cfg(target_os = "windows")]
    if std::env::var_os("WGPU_BACKEND").is_none() {
        unsafe {
            std::env::set_var("WGPU_BACKEND", "gl");
        }
        println!("[Main] WGPU backend defaulted to OpenGL (WGPU_BACKEND=gl)");
    }

    // Parse CLI arguments: expect a bundled JS file path as the first argument
    let args: Vec<String> = std::env::args().collect();
    let script_path = match args.get(1) {
        Some(path) => path.clone(),
        None => {
            eprintln!("Usage: appjs <bundle.js>");
            eprintln!("  Example: appjs ./dist/app.bundle.js");
            std::process::exit(1);
        }
    };

    // Resolve to absolute path
    let script_path = std::path::Path::new(&script_path);
    let absolute_path = match script_path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "Error: Cannot resolve script path '{}': {}",
                script_path.display(),
                e
            );
            std::process::exit(1);
        }
    };

    println!("[Main] Running script: {}", absolute_path.display());

    // Phase 1: Build the EventLoop and extract EventLoopProxy (non-blocking).
    // This must happen before spawning the JS thread so the proxy can be shared.
    let (ui_setup, event_loop) = prepare_ui();

    // Phase 2: Create IPC channels with the EventLoopProxy.
    // JS→UI commands use EventLoopProxy (immediately wakes the event loop, zero polling).
    // UI→JS events use mpsc channels.
    let channels = IpcChannels::new(ui_setup.proxy, ui_setup.window_id);

    let ui_channels = channels.ui_thread;
    let js_channels = channels.js_thread;

    // Configure the JS runtime
    let js_config = JsRuntimeConfig {
        script_path: normalize_script_path_for_bun(&absolute_path),
    };

    // Phase 3: Spawn the JS runtime thread with EventLoopProxy-based command sender.
    let js_thread_handle = thread::Builder::new()
        .name("js-runtime".to_string())
        .spawn(move || {
            println!("[Main] JS thread started");
            run_js_thread(js_channels, js_config);
            println!("[Main] JS thread finished");
        })
        .expect("Failed to spawn JS runtime thread");

    // Phase 4: Run the UI event loop on the main thread (blocks forever).
    // The main thread MUST run the UI due to platform requirements (macOS, etc.).
    println!("[Main] Starting UI on main thread");
    run_ui_blocking(event_loop, ui_setup.window_id, ui_channels.event_sender);

    // Wait for the JS thread to finish after the UI closes
    println!("[Main] UI closed, waiting for JS thread to finish...");
    if let Err(e) = js_thread_handle.join() {
        eprintln!("[Main] JS thread panicked: {:?}", e);
    }

    println!("[Main] AppJS shutdown complete");
}
