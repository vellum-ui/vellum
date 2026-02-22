import { onBridgeEvent } from "./ops.ts";
import type { VellumEvent } from "./types.ts";

type EventHandler = (event: VellumEvent) => void;

const listeners: Record<string, EventHandler[]> = {};
let eventLoopRunning = false;
let unsubscribeBridge: (() => void) | null = null;

function dispatch(event: VellumEvent): void {
    const type = event.type;
    if (!type) return;

    const handlers = listeners[type];
    if (handlers) {
        for (const handler of handlers) {
            try {
                handler(event);
            } catch (err) {
                console.error(`[Vellum] Error in '${type}' handler:`, err);
            }
        }
    }

    const wildcardHandlers = listeners["*"];
    if (wildcardHandlers) {
        for (const handler of wildcardHandlers) {
            try {
                handler(event);
            } catch (err) {
                console.error("[Vellum] Error in wildcard handler:", err);
            }
        }
    }
}

function startEventLoop(): void {
    if (eventLoopRunning) return;
    eventLoopRunning = true;
    unsubscribeBridge = onBridgeEvent((event) => dispatch(event));
}

export function on(type: string, callback: EventHandler): () => void {
    if (!listeners[type]) {
        listeners[type] = [];
    }
    listeners[type].push(callback);

    if (!eventLoopRunning) {
        startEventLoop();
    }

    return () => {
        const handlers = listeners[type];
        if (!handlers) return;
        const idx = handlers.indexOf(callback);
        if (idx >= 0) handlers.splice(idx, 1);
    };
}

export function off(type?: string): void {
    if (type) {
        delete listeners[type];
        return;
    }

    for (const key of Object.keys(listeners)) {
        delete listeners[key];
    }

    if (unsubscribeBridge) {
        unsubscribeBridge();
        unsubscribeBridge = null;
    }
    eventLoopRunning = false;
}

export const events = { on, off };
