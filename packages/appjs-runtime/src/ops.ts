import type { AppJsStyle } from "./types.ts";

type CoreOps = {
    op_set_title(title: string): void;
    op_resize_window(width: number, height: number): void;
    op_close_window(): void;
    op_create_widget(
        id: string,
        kind: string,
        parentId: string | null,
        text: string | null,
        styleJson: string | null
    ): void;
    op_remove_widget(id: string): void;
    op_set_widget_text(id: string, text: string): void;
    op_set_widget_visible(id: string, visible: boolean): void;
    op_set_widget_value(id: string, value: number): void;
    op_set_widget_checked(id: string, checked: boolean): void;
    op_set_widget_style(id: string, styleJson: string): void;
    op_set_style_property(id: string, property: string, value: string): void;
    op_log(level: string, message: string): void;
    op_exit_app(): void;
    op_wait_for_event(): Promise<string>;
};

const core = (globalThis as { Deno: { core: { ops: CoreOps } } }).Deno.core;
const ops = core.ops;

export const rawOps = ops;

export function setTitle(title: string): void {
    ops.op_set_title(title);
}

export function resizeWindow(width: number, height: number): void {
    ops.op_resize_window(width, height);
}

export function closeWindow(): void {
    ops.op_close_window();
}

export function createWidget(
    id: string,
    kind: string,
    parentId: string | null,
    text: string | null,
    style: AppJsStyle | null
): void {
    ops.op_create_widget(
        id,
        kind,
        parentId ?? null,
        text ?? null,
        style ? JSON.stringify(style) : null
    );
}

export function removeWidget(id: string): void {
    ops.op_remove_widget(id);
}

export function setWidgetText(id: string, text: string): void {
    ops.op_set_widget_text(id, text);
}

export function setWidgetVisible(id: string, visible: boolean): void {
    ops.op_set_widget_visible(id, visible);
}

export function setWidgetValue(id: string, value: number): void {
    ops.op_set_widget_value(id, value);
}

export function setWidgetChecked(id: string, checked: boolean): void {
    ops.op_set_widget_checked(id, checked);
}

export function setWidgetStyle(id: string, style: AppJsStyle): void {
    ops.op_set_widget_style(id, JSON.stringify(style));
}

export function setStyleProperty(
    id: string,
    property: string,
    value: string | number | boolean
): void {
    ops.op_set_style_property(id, property, String(value));
}

export function log(level: "debug" | "info" | "warn" | "error", message: string): void {
    ops.op_log(level, String(message));
}

export function exit(): void {
    ops.op_exit_app();
}

export function waitForEvent(): Promise<string> {
    return ops.op_wait_for_event();
}
