import type { AppJsStyle, AppJsEvent } from "./types.ts";
import {
    closeWindow,
    createWidget,
    exit,
    log as writeLog,
    removeWidget,
    resizeWindow,
    setStyleProperty,
    setTitle,
    setWidgetChecked,
    setWidgetStyle,
    setWidgetText,
    setWidgetValue,
    setWidgetVisible,
} from "./ops.ts";
import { events } from "./events.ts";

let widgetIdCounter = 0;

export function nextId(): string {
    return `__auto_${++widgetIdCounter}`;
}

export const window = {
    setTitle,
    resize: resizeWindow,
    close: closeWindow,
};

export const body = {
    setStyle: (style: AppJsStyle): void => setWidgetStyle("__root__", style),
    setStyleProperty: (property: string, value: string | number): void =>
        setStyleProperty("__root__", property, String(value)),
};

export const ui = {
    createWidget: (
        id: string,
        kind: string,
        parentId: string | null,
        text: string | null,
        style: AppJsStyle | null
    ): void => createWidget(id, kind, parentId ?? null, text ?? null, style ?? null),
    removeWidget,
    setText: setWidgetText,
    setVisible: setWidgetVisible,
    setValue: setWidgetValue,
    setChecked: setWidgetChecked,
    setStyle: setWidgetStyle,
    setStyleProperty,

    setWidgetText,
    setWidgetVisible,
};

export const log = {
    debug: (msg: string): void => writeLog("debug", msg),
    info: (msg: string): void => writeLog("info", msg),
    warn: (msg: string): void => writeLog("warn", msg),
    error: (msg: string): void => writeLog("error", msg),
};

export { events };

export function label(id: string, parentId: string | null, text: string, style?: AppJsStyle): string {
    ui.createWidget(id, "label", parentId, text, style ?? null);
    return id;
}

export function button(id: string, parentId: string | null, text: string, style?: AppJsStyle): string {
    ui.createWidget(id, "button", parentId, text, style ?? null);
    return id;
}

export function iconButton(id: string, parentId: string | null, svgData: string, style?: AppJsStyle): string {
    ui.createWidget(id, "iconButton", parentId, null, { ...style, svgData });
    return id;
}

export function svg(id: string, parentId: string | null, svgData: string, style?: AppJsStyle): string {
    ui.createWidget(id, "svg", parentId, svgData, style ?? null);
    return id;
}

export function flex(id: string, parentId: string | null, style?: AppJsStyle): string {
    ui.createWidget(id, "flex", parentId, null, style ?? null);
    return id;
}

export function row(id: string, parentId: string | null, style?: AppJsStyle): string {
    ui.createWidget(id, "flex", parentId, null, { ...style, direction: "row" });
    return id;
}

export function column(id: string, parentId: string | null, style?: AppJsStyle): string {
    ui.createWidget(id, "flex", parentId, null, { ...style, direction: "column" });
    return id;
}

export function box(id: string, parentId: string | null, style?: AppJsStyle): string {
    ui.createWidget(id, "sizedBox", parentId, null, style ?? null);
    return id;
}

export function checkbox(
    id: string,
    parentId: string | null,
    checked: boolean,
    text: string,
    style?: AppJsStyle
): string {
    ui.createWidget(id, "checkbox", parentId, text, { ...style, checked: !!checked });
    return id;
}

export function textInput(
    id: string,
    parentId: string | null,
    placeholder?: string,
    style?: AppJsStyle
): string {
    const merged = placeholder ? { ...style, placeholder } : style;
    ui.createWidget(id, "textInput", parentId, null, merged ?? null);
    return id;
}

export function prose(id: string, parentId: string | null, text?: string, style?: AppJsStyle): string {
    ui.createWidget(id, "prose", parentId, text ?? null, style ?? null);
    return id;
}

export function progressBar(
    id: string,
    parentId: string | null,
    progress?: number,
    style?: AppJsStyle
): string {
    ui.createWidget(id, "progressBar", parentId, null, { ...style, progress: progress ?? 0 });
    return id;
}

export function spinner(id: string, parentId: string | null, style?: AppJsStyle): string {
    ui.createWidget(id, "spinner", parentId, null, style ?? null);
    return id;
}

export function slider(
    id: string,
    parentId: string | null,
    min: number,
    max: number,
    value: number,
    style?: AppJsStyle
): string {
    ui.createWidget(id, "slider", parentId, null, { ...style, minValue: min, maxValue: max, progress: value });
    return id;
}

export function zstack(id: string, parentId: string | null, style?: AppJsStyle): string {
    ui.createWidget(id, "zstack", parentId, null, style ?? null);
    return id;
}

export function portal(id: string, parentId: string | null, style?: AppJsStyle): string {
    ui.createWidget(id, "portal", parentId, null, style ?? null);
    return id;
}

export { exit };
export type { AppJsStyle, AppJsEvent };

export const app = {
    window,
    body,
    ui,
    events,
    log,
    nextId,
    label,
    button,
    iconButton,
    svg,
    flex,
    row,
    column,
    box,
    checkbox,
    textInput,
    prose,
    progressBar,
    spinner,
    slider,
    zstack,
    portal,
    exit,
};

export default app;
