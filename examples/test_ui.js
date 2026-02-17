// test_ui.js -- Comprehensive UI test for AppJS
// Exercises: body styling, all widget types, nested layouts, events
import * as appjs from "../packages/appjs-runtime/src/index.ts";

// ---- Window setup ----
appjs.window.setTitle("AppJS UI Test");
appjs.window.resize(800, 700);

// ---- Body styling (root container, like <body>) ----
appjs.body.setStyle({
    background: "#1e1e2e",
    padding: 24,
    gap: 16,
    crossAxisAlignment: "fill",
});

// ---- Header ----
appjs.label("title", null, "AppJS Widget Gallery", {
    fontSize: 28,
    fontWeight: "bold",
    color: "#cdd6f4",
});

appjs.label("subtitle", null, "Testing every widget and the styling API", {
    fontSize: 14,
    color: "#a6adc8",
});

// ---- Buttons row ----
appjs.row("btn_row", null, { gap: 12 });

appjs.button("btn_primary", "btn_row", "Primary", {
    fontSize: 14,
    fontWeight: "bold",
});

appjs.button("btn_secondary", "btn_row", "Secondary", {
    fontSize: 14,
});

appjs.button("btn_click", "btn_row", "Click count: 0", {
    fontSize: 14,
});

// ---- Checkbox row ----
appjs.row("check_row", null, { gap: 16, crossAxisAlignment: "center" });
appjs.checkbox("cb1", "check_row", false, "Enable notifications");
appjs.checkbox("cb2", "check_row", true, "Dark mode");

// ---- Slider section ----
appjs.column("slider_section", null, { gap: 8, crossAxisAlignment: "fill" });
appjs.label("slider_label", "slider_section", "Volume: 50%", {
    fontSize: 14,
    color: "#bac2de",
});
appjs.slider("vol_slider", "slider_section", 0, 100, 50);

// ---- Progress bar section ----
appjs.column("progress_section", null, { gap: 8, crossAxisAlignment: "fill" });
appjs.label("progress_label", "progress_section", "Download progress:", {
    fontSize: 14,
    color: "#bac2de",
});
appjs.progressBar("dl_progress", "progress_section", 0);

// ---- Text input ----
appjs.column("input_section", null, { gap: 8, crossAxisAlignment: "fill" });
appjs.label("input_label", "input_section", "Type something:", {
    fontSize: 14,
    color: "#bac2de",
});
appjs.textInput("text_in", "input_section", "Enter text here...");

// ---- Prose (selectable read-only text) ----
appjs.prose("info_prose", null, "This is a Prose widget - you can select and copy this text. It supports multi-line content and is useful for displaying read-only information.", {
    fontSize: 13,
    color: "#9399b2",
});

// ---- Spinner (constrained to a small size) ----
appjs.row("spinner_row", null, { gap: 12, crossAxisAlignment: "center" });
appjs.box("spinner_box", "spinner_row", { width: 24, height: 24 });
appjs.spinner("loading_spinner", "spinner_box");
appjs.label("spinner_text", "spinner_row", "Loading...", {
    fontSize: 13,
    color: "#a6adc8",
});

// ---- Status label (updated by events) ----
appjs.label("status", null, "Waiting for interactions...", {
    fontSize: 12,
    color: "#585b70",
});

// ============================================================
// Event handling
// ============================================================
let clickCount = 0;

// Listen for widget actions
appjs.events.on("widgetAction", (e) => {
    const { widgetId, action, value } = e;

    if (widgetId === "btn_click" && action === "click") {
        clickCount++;
        appjs.ui.setText("btn_click", `Click count: ${clickCount}`);
        appjs.ui.setText("status", `Button clicked ${clickCount} time(s)`);

        // Update progress bar on each click (wraps at 100%)
        const progress = (clickCount % 11) / 10;
        appjs.ui.setValue("dl_progress", progress);
        const pct = Math.round(progress * 100);
        appjs.ui.setText("progress_label", `Download progress: ${pct}%`);
    }

    if (widgetId === "btn_primary" && action === "click") {
        appjs.ui.setText("status", "Primary button pressed!");
    }

    if (widgetId === "btn_secondary" && action === "click") {
        appjs.ui.setText("status", "Secondary button pressed!");
    }

    if (widgetId === "cb1" && action === "valueChanged") {
        const state = value === 1 ? "ON" : "OFF";
        appjs.ui.setText("status", `Notifications: ${state}`);
    }

    if (widgetId === "cb2" && action === "valueChanged") {
        const state = value === 1 ? "ON" : "OFF";
        appjs.ui.setText("status", `Dark mode: ${state}`);
    }

    if (widgetId === "vol_slider" && action === "valueChanged") {
        const vol = Math.round(value);
        appjs.ui.setText("slider_label", `Volume: ${vol}%`);
        appjs.ui.setText("status", `Slider moved to ${vol}%`);
    }
});

console.log("UI test loaded. Interact with the widgets!");
