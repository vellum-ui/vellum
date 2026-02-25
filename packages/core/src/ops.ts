import type { VellumStyle } from "./types.ts";
import { ensureBridge, type BridgeEvent, type Bridge, type JsToRustMessage } from "./bun_bridge.ts";

const bridge: Bridge = ensureBridge();

export const rawOps = null;

export function onBridgeEvent(callback: (event: BridgeEvent) => void): () => void {
    return bridge.onEvent(callback);
}

export function setTitle(title: string): void {
    bridge.send({ type: "setTitle", title });
}

export function resizeWindow(width: number, height: number): void {
    bridge.send({ type: "resizeWindow", width, height });
}

export function closeWindow(): void {
    bridge.send({ type: "closeWindow" });
}

export function createWidget(
    id: string,
    kind: string,
    parentId: string | null,
    text: string | null,
    style: VellumStyle | null,
    params?: object | null,
    data?: Uint8Array | null
): void {
    bridge.send({
        type: "createWidget",
        id,
        kind,
        parent_id: parentId ?? null,
        text: text ?? null,
        style_json: style ? JSON.stringify(style) : null,
        widget_params_json: params ? JSON.stringify(params) : null,
        data: data ?? null,
    });
}

export function removeWidget(id: string): void {
    bridge.send({ type: "removeWidget", id });
}

export function setWidgetText(id: string, text: string): void {
    bridge.send({ type: "setWidgetText", id, text });
}

export function setWidgetVisible(id: string, visible: boolean): void {
    bridge.send({ type: "setWidgetVisible", id, visible });
}

export function setWidgetValue(id: string, value: number): void {
    bridge.send({ type: "setWidgetValue", id, value });
}

export function setWidgetChecked(id: string, checked: boolean): void {
    bridge.send({ type: "setWidgetChecked", id, checked });
}

export function setWidgetStyle(id: string, style: VellumStyle): void {
    bridge.send({ type: "setWidgetStyle", id, style_json: JSON.stringify(style) });
}

export function setStyleProperty(
    id: string,
    property: string,
    value: string | number | boolean
): void {
    bridge.send({ type: "setStyleProperty", id, property, value: String(value) });
}

export function exit(): void {
    bridge.send({ type: "exitApp" });
}

export function setImageData(id: string, data: Uint8Array): void {
    bridge.send({ type: "setImageData", id, data });
}

export function playVideo(id: string): void {
    bridge.send({ type: "playVideo", id });
}

export function pauseVideo(id: string): void {
    bridge.send({ type: "pauseVideo", id });
}

export function seekVideo(id: string, timeSecs: number): void {
    bridge.send({ type: "seekVideo", id, time_secs: timeSecs });
}
