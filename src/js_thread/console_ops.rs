// Console ops for the JS runtime
// Provides console.log, console.warn, console.error via Deno ops

use deno_core::op2;

/// Print to stdout (used by console.log, console.info, console.debug)
#[op2(fast)]
pub fn op_console_log(#[string] msg: &str) {
    println!("{}", msg);
}

/// Print to stderr (used by console.warn)
#[op2(fast)]
pub fn op_console_warn(#[string] msg: &str) {
    eprintln!("\x1b[33m{}\x1b[0m", msg); // Yellow
}

/// Print to stderr (used by console.error)
#[op2(fast)]
pub fn op_console_error(#[string] msg: &str) {
    eprintln!("\x1b[31m{}\x1b[0m", msg); // Red
}

deno_core::extension!(
    appjs_console,
    ops = [op_console_log, op_console_warn, op_console_error],
    esm_entry_point = "ext:appjs_console/runtime.js",
    esm = ["ext:appjs_console/runtime.js" = {
        source = r#"
// Bootstrap console using ops from Rust
const core = globalThis.Deno.core;

function formatArgs(args) {
    return args.map(arg => {
        if (arg === null) return "null";
        if (arg === undefined) return "undefined";
        if (typeof arg === "string") return arg;
        if (typeof arg === "number" || typeof arg === "boolean") return String(arg);
        if (typeof arg === "symbol") return arg.toString();
        if (typeof arg === "bigint") return arg.toString() + "n";
        if (typeof arg === "function") return `[Function: ${arg.name || "anonymous"}]`;
        if (arg instanceof Error) return `${arg.constructor.name}: ${arg.message}\n${arg.stack || ""}`;
        try {
            return JSON.stringify(arg, null, 2);
        } catch {
            return String(arg);
        }
    }).join(" ");
}

globalThis.console = {
    log: (...args) => core.ops.op_console_log(formatArgs(args)),
    info: (...args) => core.ops.op_console_log(formatArgs(args)),
    debug: (...args) => core.ops.op_console_log(formatArgs(args)),
    warn: (...args) => core.ops.op_console_warn(formatArgs(args)),
    error: (...args) => core.ops.op_console_error(formatArgs(args)),
    dir: (...args) => core.ops.op_console_log(formatArgs(args)),
    trace: (...args) => {
        const err = new Error();
        core.ops.op_console_log("Trace: " + formatArgs(args) + "\n" + err.stack);
    },
    assert: (condition, ...args) => {
        if (!condition) {
            core.ops.op_console_error("Assertion failed: " + formatArgs(args));
        }
    },
    // Stubs for unimplemented methods
    table: (...args) => core.ops.op_console_log(formatArgs(args)),
    time: () => {},
    timeEnd: () => {},
    timeLog: () => {},
    clear: () => {},
    count: () => {},
    countReset: () => {},
    group: () => {},
    groupCollapsed: () => {},
    groupEnd: () => {},
};
"#
    }],
);
