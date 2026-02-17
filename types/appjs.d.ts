// AppJS Global Type Definitions
// This file provides TypeScript type information for the AppJS runtime API.
// Reference it in your .ts files with: /// <reference path="../types/appjs.d.ts" />
// Or place a tsconfig.json that includes this file.

/** Style properties accepted by widget creation and styling methods. */
interface AppJsStyle {
    // Text styles
    fontSize?: number;
    fontWeight?: number | string;
    fontStyle?: "normal" | "italic";
    fontFamily?: string;
    color?: string;
    letterSpacing?: number;
    lineHeight?: number;
    wordSpacing?: number;
    underline?: boolean;
    strikethrough?: boolean;
    textAlign?: "start" | "center" | "end" | "justify";

    // Box styles
    background?: string;
    borderColor?: string;
    borderWidth?: number;
    cornerRadius?: number;
    padding?: number | [number, number, number, number];
    width?: number;
    height?: number;

    // Flex styles
    direction?: "row" | "column";
    crossAxisAlignment?: "start" | "center" | "end" | "fill" | "baseline";
    mainAxisAlignment?: "start" | "center" | "end" | "spaceBetween" | "spaceAround" | "spaceEvenly";
    gap?: number;
    flex?: number;
    mustFillMainAxis?: boolean;

    // Widget-specific
    checked?: boolean;
    progress?: number;
    placeholder?: string;
    min?: number;
    minValue?: number;
    max?: number;
    maxValue?: number;
    step?: number;

    // Allow additional properties
    [key: string]: unknown;
}

/** Event object dispatched by the AppJS event system. */
interface AppJsEvent {
    type: string;
    widgetId?: string;
    action?: string;
    value?: string | number | boolean;
    width?: number;
    height?: number;
    x?: number;
    y?: number;
    key?: string;
    text?: string;
    focused?: boolean;
}

/** Window management API. */
interface AppJsWindow {
    /** Set the window title. */
    setTitle(title: string): void;
    /** Resize the window. */
    resize(width: number, height: number): void;
    /** Close the window. */
    close(): void;
}

/** Root body (root container) styling API. */
interface AppJsBody {
    /** Set full style on the root container. */
    setStyle(style: AppJsStyle): void;
    /** Set a single style property on the root container. */
    setStyleProperty(property: string, value: string | number): void;
}

/** Low-level UI / widget management API. */
interface AppJsUi {
    /**
     * Create a widget.
     * @param id - Unique widget ID
     * @param kind - Widget type: "label", "button", "flex", "container",
     *   "sizedBox", "checkbox", "textInput", "textArea", "prose", "progressBar",
     *   "spinner", "slider", "grid", "zstack", "portal"
     * @param parentId - Parent widget ID (null = root)
     * @param text - Initial text content
     * @param style - Style object
     */
    createWidget(id: string, kind: string, parentId: string | null, text: string | null, style: AppJsStyle | null): void;
    /** Remove a widget by ID. */
    removeWidget(id: string): void;
    /** Set a widget's text content. */
    setText(id: string, text: string): void;
    /** Set a widget's visibility. */
    setVisible(id: string, visible: boolean): void;
    /** Set a numeric value (e.g., progress bar, slider). */
    setValue(id: string, value: number): void;
    /** Set checkbox checked state. */
    setChecked(id: string, checked: boolean): void;
    /** Set full style object on an existing widget. */
    setStyle(id: string, style: AppJsStyle): void;
    /** Set a single style property on a widget. */
    setStyleProperty(id: string, property: string, value: string | number | boolean): void;
    /** @deprecated Use setText instead. */
    setWidgetText(id: string, text: string): void;
    /** @deprecated Use setVisible instead. */
    setWidgetVisible(id: string, visible: boolean): void;
}

/** Event system API. */
interface AppJsEvents {
    /**
     * Register a listener for a UI event type.
     * Supported types: "windowResized", "mouseClick", "mouseMove", "keyPress",
     *   "keyRelease", "textInput", "widgetAction", "windowFocusChanged",
     *   "windowCloseRequested", "appExit"
     * Use "*" to listen for all events.
     * @returns An unsubscribe function.
     */
    on(type: string, callback: (event: AppJsEvent) => void): () => void;
    /**
     * Remove all listeners for a specific event type, or all listeners.
     * @param type - If provided, only remove listeners for this type.
     */
    off(type?: string): void;
}

/** Logging API. */
interface AppJsLog {
    debug(msg: string): void;
    info(msg: string): void;
    warn(msg: string): void;
    error(msg: string): void;
}

/** The main AppJS API available as `globalThis.appjs`. */
interface AppJs {
    window: AppJsWindow;
    body: AppJsBody;
    ui: AppJsUi;
    events: AppJsEvents;
    log: AppJsLog;

    // ---- Convenience widget creation methods ----

    /** Create a Label widget. */
    label(id: string, parentId: string | null, text: string, style?: AppJsStyle): string;
    /** Create a Button widget. */
    button(id: string, parentId: string | null, text: string, style?: AppJsStyle): string;
    /** Create a Flex container (column by default). */
    flex(id: string, parentId: string | null, style?: AppJsStyle): string;
    /** Create a row Flex container. */
    row(id: string, parentId: string | null, style?: AppJsStyle): string;
    /** Create a column Flex container. */
    column(id: string, parentId: string | null, style?: AppJsStyle): string;
    /** Create a SizedBox (box with fixed dimensions). */
    box(id: string, parentId: string | null, style?: AppJsStyle): string;
    /** Create a Checkbox. */
    checkbox(id: string, parentId: string | null, checked: boolean, text: string, style?: AppJsStyle): string;
    /** Create a TextInput. */
    textInput(id: string, parentId: string | null, placeholder?: string, style?: AppJsStyle): string;
    /** Create a Prose (selectable read-only text). */
    prose(id: string, parentId: string | null, text?: string, style?: AppJsStyle): string;
    /** Create a ProgressBar. */
    progressBar(id: string, parentId: string | null, progress?: number, style?: AppJsStyle): string;
    /** Create a Spinner (loading indicator). */
    spinner(id: string, parentId: string | null, style?: AppJsStyle): string;
    /** Create a Slider. */
    slider(id: string, parentId: string | null, min: number, max: number, value: number, style?: AppJsStyle): string;
    /** Create a ZStack (overlay container). */
    zstack(id: string, parentId: string | null, style?: AppJsStyle): string;
    /** Create a Portal (scrollable container). */
    portal(id: string, parentId: string | null, style?: AppJsStyle): string;

    /** Exit the application. */
    exit(): void;
}

declare const appjs: AppJs;
