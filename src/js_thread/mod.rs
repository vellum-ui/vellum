// JS Thread Module
// Handles the JavaScript runtime execution using deno_core

mod console_ops;
pub mod ipc_ops;

use std::rc::Rc;
use std::sync::{Arc, Mutex};

use deno_core::{FsModuleLoader, JsRuntime, ModuleSpecifier, RuntimeOptions};

use crate::ipc::{JsCommand, JsThreadChannels, LogLevel};

/// Configuration for the JS runtime
pub struct JsRuntimeConfig {
    /// Path to the main JavaScript module to execute
    pub main_module_path: String,
}

impl Default for JsRuntimeConfig {
    fn default() -> Self {
        Self {
            main_module_path: "./main.js".to_string(),
        }
    }
}

/// Run the JS runtime on a background thread
///
/// This function creates a new tokio runtime and runs the JS event loop.
/// It should be called from `std::thread::spawn`.
pub fn run_js_thread(channels: JsThreadChannels, config: JsRuntimeConfig) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");

    rt.block_on(async move {
        if let Err(e) = run_js_runtime(channels, config).await {
            eprintln!("[JS] Runtime error: {:?}", e);
        }
    });
}

/// The async inner function that sets up and runs the JS runtime
async fn run_js_runtime(
    channels: JsThreadChannels,
    config: JsRuntimeConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let command_sender = channels.command_sender;
    let event_receiver = channels.event_receiver;

    // Helper to log via IPC
    let log = |msg: &str| {
        let _ = command_sender.send(JsCommand::Log {
            level: LogLevel::Info,
            message: msg.to_string(),
        });
    };

    log("Initializing JS runtime...");

    // Resolve the module path to a file:// URL
    let module_path = std::path::Path::new(&config.main_module_path);
    let main_module = ModuleSpecifier::from_file_path(module_path)
        .map_err(|_| format!("Invalid module path: {}", config.main_module_path))?;

    log(&format!("Loading module: {}", main_module));

    // Prepare the IPC extension with state injected into OpState
    let shared_receiver = ipc_ops::SharedEventReceiver(Arc::new(Mutex::new(event_receiver)));
    let sender_for_state = command_sender.clone();

    let mut ipc_ext = ipc_ops::appjs_ipc::init();
    ipc_ext.op_state_fn = Some(Box::new(move |state| {
        state.put(sender_for_state);
        state.put(shared_receiver);
    }));

    // Create the deno_core JsRuntime
    let mut runtime = JsRuntime::new(RuntimeOptions {
        module_loader: Some(Rc::new(FsModuleLoader)),
        extensions: vec![
            console_ops::appjs_console::init(),
            ipc_ext,
        ],
        ..Default::default()
    });

    log("JS runtime initialized, executing module...");

    // Load and evaluate the main module
    let mod_id = runtime
        .load_main_es_module(&main_module)
        .await
        .map_err(|e| format!("Failed to load module '{}': {}", main_module, e))?;

    let result = runtime.mod_evaluate(mod_id);

    // Run the event loop — this processes the module evaluation and any async ops
    // (including the event listener loop if the user registered any listeners via appjs.events.on())
    runtime
        .run_event_loop(Default::default())
        .await
        .map_err(|e| format!("Event loop error: {}", e))?;

    // Await the module evaluation result
    result
        .await
        .map_err(|e| format!("Module evaluation error: {}", e))?;

    log("JS runtime finished");

    Ok(())
}
