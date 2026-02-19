import process from "node:process";
import { decode, encode } from "@msgpack/msgpack";

export type BridgeEvent = {
    type: string;
    widgetId?: string;
    action?: string;
    value?: string | number | boolean;
};

export type JsToRustMessage =
    | { type: "setTitle"; title: string }
    | {
        type: "createWidget";
        id: string;
        kind: string;
        parent_id: string | null;
        text: string | null;
        style_json: string | null;
    }
    | { type: "removeWidget"; id: string }
    | { type: "setWidgetText"; id: string; text: string }
    | { type: "setWidgetVisible"; id: string; visible: boolean }
    | { type: "setWidgetValue"; id: string; value: number }
    | { type: "setWidgetChecked"; id: string; checked: boolean }
    | { type: "setWidgetStyle"; id: string; style_json: string }
    | { type: "setStyleProperty"; id: string; property: string; value: string }
    | { type: "resizeWindow"; width: number; height: number }
    | { type: "closeWindow" }
    | { type: "log"; level: string; message: string }
    | { type: "exitApp" }
    | { type: "ready" };

type RustToJsMessage =
    | { type: "uiEvent"; event: unknown }
    | { type: "shutdown" };

export type Bridge = {
    send(message: JsToRustMessage): void;
    onEvent(callback: (event: BridgeEvent) => void): () => void;
};

type AppJsGlobal = typeof globalThis & {
    __APPJS_BRIDGE__?: Bridge;
    __APPJS_CONSOLE_PATCHED__?: boolean;
};

function writeFrame(message: JsToRustMessage): void {
    const payload = Buffer.from(encode(message));
    const frame = Buffer.allocUnsafe(4 + payload.length);
    frame.writeUInt32LE(payload.length, 0);
    payload.copy(frame, 4);
    process.stdout.write(frame);
}

function mapUiEvent(event: unknown): BridgeEvent {
    const widgetAction = (event as { WidgetAction?: { widget_id?: string; action?: unknown } })?.WidgetAction;
    if (!widgetAction) {
        return { type: "unknown" };
    }

    if (widgetAction.action === "Click") {
        return {
            type: "widgetAction",
            widgetId: widgetAction.widget_id,
            action: "click",
        };
    }

    const valueChanged = (widgetAction.action as { ValueChanged?: number } | undefined)?.ValueChanged;
    if (valueChanged !== undefined) {
        return {
            type: "widgetAction",
            widgetId: widgetAction.widget_id,
            action: "valueChanged",
            value: valueChanged,
        };
    }

    return { type: "unknown" };
}

function formatLogArgs(args: unknown[]): string {
    return args
        .map((arg) => {
            if (typeof arg === "string") return arg;
            if (typeof arg === "number" || typeof arg === "boolean" || typeof arg === "bigint") {
                return String(arg);
            }
            if (arg instanceof Error) {
                return `${arg.name}: ${arg.message}\n${arg.stack ?? ""}`;
            }
            try {
                return JSON.stringify(arg);
            } catch {
                return String(arg);
            }
        })
        .join(" ");
}

function patchConsole(bridge: Bridge): void {
    const globalScope = globalThis as AppJsGlobal;
    if (globalScope.__APPJS_CONSOLE_PATCHED__) return;
    globalScope.__APPJS_CONSOLE_PATCHED__ = true;

    const send = (level: "debug" | "info" | "warn" | "error", args: unknown[]) => {
        try {
            bridge.send({ type: "log", level, message: formatLogArgs(args) });
        } catch {
            process.stderr.write(`${formatLogArgs(args)}\n`);
        }
    };

    console.log = (...args: unknown[]) => send("info", args);
    console.info = (...args: unknown[]) => send("info", args);
    console.debug = (...args: unknown[]) => send("debug", args);
    console.warn = (...args: unknown[]) => send("warn", args);
    console.error = (...args: unknown[]) => send("error", args);
}

export function initAppJsBridge(): Bridge {
    const globalScope = globalThis as AppJsGlobal;
    if (globalScope.__APPJS_BRIDGE__) {
        return globalScope.__APPJS_BRIDGE__;
    }

    const listeners = new Set<(event: BridgeEvent) => void>();
    let readBuffer = Buffer.alloc(0);

    const bridge: Bridge = {
        send(message) {
            writeFrame(message);
        },
        onEvent(callback) {
            listeners.add(callback);
            return () => {
                listeners.delete(callback);
            };
        },
    };

    const emitEvent = (event: BridgeEvent) => {
        for (const listener of listeners) {
            try {
                listener(event);
            } catch (err) {
                process.stderr.write(`[appjs bridge] Event listener error: ${String(err)}\n`);
            }
        }
    };

    const handleFrame = (frame: Buffer) => {
        try {
            const message = decode(frame) as RustToJsMessage;
            if (message?.type === "uiEvent") {
                emitEvent(mapUiEvent(message.event));
                return;
            }
            if (message?.type === "shutdown") {
                process.exit(0);
            }
        } catch (err) {
            process.stderr.write(`[appjs bridge] Decode error: ${String(err)}\n`);
        }
    };

    const processReadBuffer = () => {
        while (readBuffer.length >= 4) {
            const len = readBuffer.readUInt32LE(0);
            if (readBuffer.length < 4 + len) {
                return;
            }

            const frame = readBuffer.subarray(4, 4 + len);
            readBuffer = readBuffer.subarray(4 + len);
            handleFrame(frame);
        }
    };

    process.stdin.on("data", (chunk) => {
        readBuffer = Buffer.concat([readBuffer, Buffer.from(chunk)]);
        processReadBuffer();
    });

    process.stdin.on("end", () => {
        process.exit(0);
    });

    process.stdin.resume();

    globalScope.__APPJS_BRIDGE__ = bridge;
    patchConsole(bridge);
    bridge.send({ type: "ready" });
    return bridge;
}

export function ensureBridge(): Bridge {
    return initAppJsBridge();
}

initAppJsBridge();
