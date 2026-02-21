import type {
    AppJsEvent,
    AppJsStyle,
    BoxStyle,
    ButtonParams,
    CheckboxParams,
    ImageParams,
    ProgressBarParams,
    SliderParams,
    SvgParams,
    TextInputParams,
} from "./types.ts";
import {
    closeWindow,
    createWidget,
    exit,
    log as writeLog,
    removeWidget,
    resizeWindow,
    setImageData,
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
        style: BoxStyle | null,
        params?: object | null,
        data?: Uint8Array | null
    ): void => createWidget(id, kind, parentId ?? null, text ?? null, style ?? null, params ?? null, data ?? null),
    removeWidget,
    setText: setWidgetText,
    setVisible: setWidgetVisible,
    setValue: setWidgetValue,
    setChecked: setWidgetChecked,
    setStyle: setWidgetStyle,
    setStyleProperty,
    setImageData,

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
export type * from "./widgets/types.ts";

export function label(id: string, parentId: string | null, text: string, style?: AppJsStyle): string {
    ui.createWidget(id, "label", parentId, text, style ?? null);
    return id;
}

export function button(id: string, parentId: string | null, text: string, style?: AppJsStyle): string {
    ui.createWidget(id, "button", parentId, text, style ?? null);
    return id;
}

export function iconButton(id: string, parentId: string | null, svgData: string, style?: AppJsStyle): string {
    const params: ButtonParams = { svgData };
    ui.createWidget(id, "iconButton", parentId, null, style ?? null, params);
    return id;
}

export function svg(id: string, parentId: string | null, svgData: string, style?: AppJsStyle): string {
    const params: SvgParams = { svgData };
    ui.createWidget(id, "svg", parentId, null, style ?? null, params);
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
    const params: CheckboxParams = { checked: !!checked };
    ui.createWidget(id, "checkbox", parentId, text, style ?? null, params);
    return id;
}

export function textInput(
    id: string,
    parentId: string | null,
    placeholder?: string,
    style?: AppJsStyle
): string {
    const params: TextInputParams | null = placeholder ? { placeholder } : null;
    ui.createWidget(id, "textInput", parentId, null, style ?? null, params);
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
    const params: ProgressBarParams = { progress: progress ?? 0 };
    ui.createWidget(id, "progressBar", parentId, null, style ?? null, params);
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
    style?: AppJsStyle,
    step?: number,
): string {
    const params: SliderParams = {
        minValue: min,
        maxValue: max,
        value,
        ...(step !== undefined ? { step } : {}),
    };
    ui.createWidget(id, "slider", parentId, null, style ?? null, params);
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
export type { AppJsStyle, AppJsEvent, BoxStyle };

export function image(
    id: string,
    parentId: string | null,
    data: Uint8Array,
    style?: AppJsStyle & { objectFit?: string }
): string {
    const objectFit = style?.objectFit;
    const { objectFit: _of, ...restStyle } = style ?? {};
    const paramsJson: ImageParams | null = objectFit ? { object_fit: objectFit } : null;
    ui.createWidget(
        id,
        "image",
        parentId,
        null,
        Object.keys(restStyle).length > 0 ? restStyle : null,
        paramsJson,
        data
    );
    return id;
}

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
    image,
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
