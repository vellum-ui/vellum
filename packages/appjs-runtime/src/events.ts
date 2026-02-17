import { waitForEvent } from "./ops.ts";
import type { AppJsEvent } from "./types.ts";

type EventHandler = (event: AppJsEvent) => void;

const listeners: Record<string, EventHandler[]> = {};
let eventLoopRunning = false;

function dispatch(eventJson: string): void {
    const event = JSON.parse(eventJson) as AppJsEvent;
    const type = event.type;
    if (!type) return;

    const handlers = listeners[type];
    if (handlers) {
        for (const handler of handlers) {
            try {
                handler(event);
            } catch (err) {
                console.error(`[appjs] Error in '${type}' handler:`, err);
            }
        }
    }

    const wildcardHandlers = listeners["*"];
    if (wildcardHandlers) {
        for (const handler of wildcardHandlers) {
            try {
                handler(event);
            } catch (err) {
                console.error("[appjs] Error in wildcard handler:", err);
            }
        }
    }
}

async function startEventLoop(): Promise<void> {
    if (eventLoopRunning) return;
    eventLoopRunning = true;

    while (eventLoopRunning) {
        try {
            const eventJson = await waitForEvent();
            if (!eventJson) {
                eventLoopRunning = false;
                break;
            }

            const parsed = JSON.parse(eventJson) as AppJsEvent;
            if (parsed.type === "disconnected") {
                eventLoopRunning = false;
                break;
            }

            dispatch(eventJson);
        } catch (err) {
            console.error("[appjs] Event loop error:", err);
            eventLoopRunning = false;
            break;
        }
    }
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
}

export const events = { on, off };
