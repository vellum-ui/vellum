// AppJS IPC Bridge -- JavaScript API
// Exposes globalThis.appjs for controlling the UI and listening for events
const core = globalThis.Deno.core;

// ============================================================
// Event emitter internals
// ============================================================
const _listeners = {};
let _eventLoopRunning = false;

function _dispatch(eventJson) {
    const event = JSON.parse(eventJson);
    const type = event.type;
    if (!type) return;

    const handlers = _listeners[type];
    if (handlers) {
        for (const handler of handlers) {
            try {
                handler(event);
            } catch (err) {
                console.error(`[appjs] Error in '${type}' handler:`, err);
            }
        }
    }

    // Also dispatch to wildcard listeners
    const wildcardHandlers = _listeners["*"];
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

async function _startEventLoop() {
    if (_eventLoopRunning) return;
    _eventLoopRunning = true;

    while (_eventLoopRunning) {
        try {
            const eventJson = await core.ops.op_wait_for_event();
            if (!eventJson) {
                _eventLoopRunning = false;
                break;
            }

            const parsed = JSON.parse(eventJson);
            if (parsed.type === "disconnected") {
                _eventLoopRunning = false;
                break;
            }

            _dispatch(eventJson);
        } catch (err) {
            console.error("[appjs] Event loop error:", err);
            _eventLoopRunning = false;
            break;
        }
    }
}

// ============================================================
// Internal: auto-increment widget IDs
// ============================================================
let _widgetIdCounter = 0;
function _nextId() {
    return `__auto_${++_widgetIdCounter}`;
}

// ============================================================
// Public API: globalThis.appjs
// ============================================================
globalThis.appjs = {
    // ---- Window management ----
    window: {
        setTitle: (title) => core.ops.op_set_title(title),
        resize: (width, height) => core.ops.op_resize_window(width, height),
        close: () => core.ops.op_close_window(),
    },

    // ---- Body (root element) ----
    /**
     * The root container - like <body> in HTML.
     * Style it to control the window's background, padding, layout, etc.
     *
     * Supported style properties:
     *   background, padding, gap, crossAxisAlignment, mainAxisAlignment,
     *   mustFillMainAxis, borderColor, borderWidth, cornerRadius, color
     *
     * Example:
     *   appjs.body.setStyle({ background: "#1e1e2e", padding: 20, gap: 12 });
     */
    body: {
        /** Set full style on the root container */
        setStyle: (style) =>
            core.ops.op_set_widget_style("__root__", JSON.stringify(style)),
        /** Set a single style property on the root container */
        setStyleProperty: (property, value) =>
            core.ops.op_set_style_property("__root__", property, String(value)),
    },

    // ---- UI / Widget management ----
    ui: {
        /**
         * Create a widget.
         * @param {string} id - Unique widget ID
         * @param {string} kind - Widget type: "label", "button", "flex", "container",
         *   "sizedBox", "checkbox", "textInput", "textArea", "prose", "progressBar",
         *   "spinner", "slider", "grid", "zstack", "portal", "scroll"
         * @param {string|null} parentId - Parent widget ID (null = root)
         * @param {string|null} text - Initial text content
         * @param {object|null} style - Style object (see styling docs)
         */
        createWidget: (id, kind, parentId, text, style) =>
            core.ops.op_create_widget(
                id,
                kind,
                parentId ?? null,
                text ?? null,
                style ? JSON.stringify(style) : null
            ),
        removeWidget: (id) => core.ops.op_remove_widget(id),
        setText: (id, text) => core.ops.op_set_widget_text(id, text),
        setVisible: (id, visible) => core.ops.op_set_widget_visible(id, visible),
        setValue: (id, value) => core.ops.op_set_widget_value(id, value),
        setChecked: (id, checked) => core.ops.op_set_widget_checked(id, checked),

        /**
         * Set full style object on an existing widget
         * @param {string} id - Widget ID
         * @param {object} style - Style object
         */
        setStyle: (id, style) =>
            core.ops.op_set_widget_style(id, JSON.stringify(style)),

        /**
         * Set a single style property
         * @param {string} id - Widget ID
         * @param {string} property - Property name (camelCase)
         * @param {string|number|boolean} value - Property value
         */
        setStyleProperty: (id, property, value) =>
            core.ops.op_set_style_property(id, property, String(value)),

        // Legacy alias
        setWidgetText: (id, text) => core.ops.op_set_widget_text(id, text),
        setWidgetVisible: (id, visible) => core.ops.op_set_widget_visible(id, visible),
    },

    // ---- Convenience widget creation methods ----

    /**
     * Create a Label widget
     * @param {string} id - Unique widget ID
     * @param {string|null} parentId - Parent widget ID (null = root)
     * @param {string} text - Label text
     * @param {object} [style] - Style object
     * @returns {string} widget ID
     */
    label: (id, parentId, text, style) => {
        appjs.ui.createWidget(id, "label", parentId, text, style);
        return id;
    },

    /**
     * Create a Button widget
     * @param {string} id - Unique widget ID
     * @param {string|null} parentId - Parent widget ID (null = root)
     * @param {string} text - Button label text
     * @param {object} [style] - Style object
     * @returns {string} widget ID
     */
    button: (id, parentId, text, style) => {
        appjs.ui.createWidget(id, "button", parentId, text, style);
        return id;
    },

    /**
     * Create a Flex container (column by default)
     * @param {string} id - Unique widget ID
     * @param {string|null} parentId - Parent widget ID (null = root)
     * @param {object} [style] - Style object (use direction: "row"/"column")
     * @returns {string} widget ID
     */
    flex: (id, parentId, style) => {
        appjs.ui.createWidget(id, "flex", parentId, null, style);
        return id;
    },

    /**
     * Create a row Flex container
     * @param {string} id - Unique widget ID
     * @param {string|null} parentId - Parent widget ID (null = root)
     * @param {object} [style] - Style object
     * @returns {string} widget ID
     */
    row: (id, parentId, style) => {
        appjs.ui.createWidget(id, "flex", parentId, null, { ...style, direction: "row" });
        return id;
    },

    /**
     * Create a column Flex container
     * @param {string} id - Unique widget ID
     * @param {string|null} parentId - Parent widget ID (null = root)
     * @param {object} [style] - Style object
     * @returns {string} widget ID
     */
    column: (id, parentId, style) => {
        appjs.ui.createWidget(id, "flex", parentId, null, { ...style, direction: "column" });
        return id;
    },

    /**
     * Create a SizedBox (box with fixed dimensions)
     * @param {string} id - Unique widget ID
     * @param {string|null} parentId - Parent widget ID (null = root)
     * @param {object} [style] - Style object (use width/height)
     * @returns {string} widget ID
     */
    box: (id, parentId, style) => {
        appjs.ui.createWidget(id, "sizedBox", parentId, null, style);
        return id;
    },

    /**
     * Create a Checkbox
     * @param {string} id - Unique widget ID
     * @param {string|null} parentId - Parent widget ID (null = root)
     * @param {boolean} checked - Initial checked state
     * @param {string} text - Checkbox label text
     * @param {object} [style] - Style object
     * @returns {string} widget ID
     */
    checkbox: (id, parentId, checked, text, style) => {
        const s = { ...style, checked: !!checked };
        appjs.ui.createWidget(id, "checkbox", parentId, text, s);
        return id;
    },

    /**
     * Create a TextInput
     * @param {string} id - Unique widget ID
     * @param {string|null} parentId - Parent widget ID (null = root)
     * @param {string} [placeholder] - Placeholder text
     * @param {object} [style] - Style object
     * @returns {string} widget ID
     */
    textInput: (id, parentId, placeholder, style) => {
        const s = placeholder ? { ...style, placeholder } : style;
        appjs.ui.createWidget(id, "textInput", parentId, null, s);
        return id;
    },

    /**
     * Create a Prose (selectable read-only text)
     * @param {string} id - Unique widget ID
     * @param {string|null} parentId - Parent widget ID (null = root)
     * @param {string} [text] - Text content
     * @param {object} [style] - Style object
     * @returns {string} widget ID
     */
    prose: (id, parentId, text, style) => {
        appjs.ui.createWidget(id, "prose", parentId, text, style);
        return id;
    },

    /**
     * Create a ProgressBar
     * @param {string} id - Unique widget ID
     * @param {string|null} parentId - Parent widget ID (null = root)
     * @param {number} [progress=0] - Progress 0.0 to 1.0
     * @param {object} [style] - Style object
     * @returns {string} widget ID
     */
    progressBar: (id, parentId, progress, style) => {
        const s = { ...style, progress: progress ?? 0 };
        appjs.ui.createWidget(id, "progressBar", parentId, null, s);
        return id;
    },

    /**
     * Create a Spinner (loading indicator)
     * @param {string} id - Unique widget ID
     * @param {string|null} parentId - Parent widget ID (null = root)
     * @param {object} [style] - Style object
     * @returns {string} widget ID
     */
    spinner: (id, parentId, style) => {
        appjs.ui.createWidget(id, "spinner", parentId, null, style);
        return id;
    },

    /**
     * Create a Slider
     * @param {string} id - Unique widget ID
     * @param {string|null} parentId - Parent widget ID (null = root)
     * @param {number} min - Minimum value
     * @param {number} max - Maximum value
     * @param {number} value - Initial value
     * @param {object} [style] - Style object
     * @returns {string} widget ID
     */
    slider: (id, parentId, min, max, value, style) => {
        const s = { ...style, minValue: min, maxValue: max, progress: value };
        appjs.ui.createWidget(id, "slider", parentId, null, s);
        return id;
    },

    /**
     * Create a ZStack (overlay container)
     * @param {string} id - Unique widget ID
     * @param {string|null} parentId - Parent widget ID (null = root)
     * @param {object} [style] - Style object
     * @returns {string} widget ID
     */
    zstack: (id, parentId, style) => {
        appjs.ui.createWidget(id, "zstack", parentId, null, style);
        return id;
    },

    /**
     * Create a Portal (scrollable container)
     * @param {string} id - Unique widget ID
     * @param {string|null} parentId - Parent widget ID (null = root)
     * @param {object} [style] - Style object
     * @returns {string} widget ID
     */
    portal: (id, parentId, style) => {
        appjs.ui.createWidget(id, "portal", parentId, null, style);
        return id;
    },

    // ---- Event system ----
    events: {
        /**
         * Register a listener for a UI event type.
         * Supported types: windowResized, mouseClick, mouseMove, keyPress,
         *   keyRelease, textInput, widgetAction, windowFocusChanged,
         *   windowCloseRequested, appExit
         * Use "*" to listen for all events.
         *
         * @param {string} type - Event type name
         * @param {function} callback - Handler function receiving the event object
         * @returns {function} unsubscribe function
         */
        on: (type, callback) => {
            if (!_listeners[type]) {
                _listeners[type] = [];
            }
            _listeners[type].push(callback);

            // Auto-start the event loop on first listener registration
            if (!_eventLoopRunning) {
                _startEventLoop();
            }

            // Return unsubscribe function
            return () => {
                const handlers = _listeners[type];
                if (handlers) {
                    const idx = handlers.indexOf(callback);
                    if (idx >= 0) handlers.splice(idx, 1);
                }
            };
        },

        /**
         * Remove all listeners for a specific event type, or all listeners.
         * @param {string} [type] - If provided, only remove listeners for this type
         */
        off: (type) => {
            if (type) {
                delete _listeners[type];
            } else {
                for (const key of Object.keys(_listeners)) {
                    delete _listeners[key];
                }
            }
        },
    },

    // ---- Logging ----
    log: {
        debug: (msg) => core.ops.op_log("debug", String(msg)),
        info: (msg) => core.ops.op_log("info", String(msg)),
        warn: (msg) => core.ops.op_log("warn", String(msg)),
        error: (msg) => core.ops.op_log("error", String(msg)),
    },

    // ---- App lifecycle ----
    exit: () => core.ops.op_exit_app(),
};

// ============================================================
// Style helper reference (for documentation)
// ============================================================
// Style properties accepted by all widget creation methods:
//
// Text styles:
//   fontSize: number         - Font size in px (e.g., 16)
//   fontWeight: number|string - Weight (100-900, or "bold", "normal", etc.)
//   fontStyle: string        - "normal" or "italic"
//   fontFamily: string       - Font family name
//   color: string            - Text color ("#RRGGBB", "#RRGGBBAA", "rgb(r,g,b)", named)
//   letterSpacing: number    - Letter spacing in px
//   lineHeight: number       - Line height multiplier
//   wordSpacing: number      - Word spacing in px
//   underline: boolean       - Enable underline
//   strikethrough: boolean   - Enable strikethrough
//   textAlign: string        - "start", "center", "end", "justify"
//
// Box styles:
//   background: string       - Background color
//   borderColor: string      - Border color
//   borderWidth: number      - Border width in px
//   cornerRadius: number     - Corner radius in px
//   padding: number|array    - Uniform or [top, right, bottom, left]
//   width: number            - Fixed width
//   height: number           - Fixed height
//
// Flex styles:
//   direction: string        - "row" or "column"
//   crossAxisAlignment: string - "start", "center", "end", "fill", "baseline"
//   mainAxisAlignment: string  - "start", "center", "end", "spaceBetween", "spaceAround", "spaceEvenly"
//   gap: number              - Gap between children
//   flex: number             - Flex grow factor
//   mustFillMainAxis: boolean - Whether to fill main axis
//
// Widget-specific:
//   checked: boolean         - Checkbox initial state
//   progress: number         - ProgressBar value (0.0-1.0)
//   placeholder: string      - TextInput placeholder
//   min/minValue: number     - Slider minimum
//   max/maxValue: number     - Slider maximum
//   step: number             - Slider step size
