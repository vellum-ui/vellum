// test_ui.js -- Comprehensive UI test for Vellum UI
// Exercises: body styling, all widget types, nested layouts, events
import * as Vellum UI from "@vellum/core";

// ---- Window setup ----
Vellum UI.window.setTitle("Vellum UI UI Test");
Vellum UI.window.resize(800, 700);

// ---- Body styling (root container, like <body>) ----
Vellum UI.body.setStyle({
    background: "#1e1e2e",
    padding: 24,
    gap: 16,
    crossAxisAlignment: "fill",
});

// ---- Header ----
Vellum UI.label("title", null, "Vellum UI Widget Gallery", {
    fontSize: 28,
    fontWeight: "bold",
    color: "#cdd6f4",
});

Vellum UI.label("subtitle", null, "Testing every widget and the styling API", {
    fontSize: 14,
    color: "#a6adc8",
});

// ---- Buttons row ----
Vellum UI.row("btn_row", null, { gap: 12 });

Vellum UI.button("btn_primary", "btn_row", "Primary", {
    fontSize: 14,
    fontWeight: "bold",
});

Vellum UI.button("btn_secondary", "btn_row", "Secondary", {
    fontSize: 14,
});

Vellum UI.button("btn_click", "btn_row", "Click count: 0", {
    fontSize: 14,
});

// ---- Checkbox row ----
Vellum UI.row("check_row", null, { gap: 16, crossAxisAlignment: "center" });
Vellum UI.checkbox("cb1", "check_row", false, "Enable notifications");
Vellum UI.checkbox("cb2", "check_row", true, "Dark mode");

// ---- Slider section ----
Vellum UI.column("slider_section", null, { gap: 8, crossAxisAlignment: "fill" });
Vellum UI.label("slider_label", "slider_section", "Volume: 50%", {
    fontSize: 14,
    color: "#bac2de",
});
Vellum UI.slider("vol_slider", "slider_section", 0, 100, 50);

// ---- Progress bar section ----
Vellum UI.column("progress_section", null, { gap: 8, crossAxisAlignment: "fill" });
Vellum UI.label("progress_label", "progress_section", "Download progress:", {
    fontSize: 14,
    color: "#bac2de",
});
Vellum UI.progressBar("dl_progress", "progress_section", 0);

// ---- Text input ----
Vellum UI.column("input_section", null, { gap: 8, crossAxisAlignment: "fill" });
Vellum UI.label("input_label", "input_section", "Type something:", {
    fontSize: 14,
    color: "#bac2de",
});
Vellum UI.textInput("text_in", "input_section", "Enter text here...");

// ---- Prose (selectable read-only text) ----
Vellum UI.prose("info_prose", null, "This is a Prose widget - you can select and copy this text. It supports multi-line content and is useful for displaying read-only information.", {
    fontSize: 13,
    color: "#9399b2",
});

// ---- Spinner (constrained to a small size) ----
Vellum UI.row("spinner_row", null, { gap: 12, crossAxisAlignment: "center" });
Vellum UI.box("spinner_box", "spinner_row", { width: 24, height: 24 });
Vellum UI.spinner("loading_spinner", "spinner_box");
Vellum UI.label("spinner_text", "spinner_row", "Loading...", {
    fontSize: 13,
    color: "#a6adc8",
});

// ---- Status label (updated by events) ----
Vellum UI.label("status", null, "Waiting for interactions...", {
    fontSize: 12,
    color: "#585b70",
});

// ============================================================
// Event handling
// ============================================================
let clickCount = 0;

// Listen for widget actions
Vellum UI.events.on("widgetAction", (e) => {
    const { widgetId, action, value } = e;

    if (widgetId === "btn_click" && action === "click") {
        clickCount++;
        Vellum UI.ui.setText("btn_click", `Click count: ${clickCount}`);
        Vellum UI.ui.setText("status", `Button clicked ${clickCount} time(s)`);

        // Update progress bar on each click (wraps at 100%)
        const progress = (clickCount % 11) / 10;
        Vellum UI.ui.setValue("dl_progress", progress);
        const pct = Math.round(progress * 100);
        Vellum UI.ui.setText("progress_label", `Download progress: ${pct}%`);
    }

    if (widgetId === "btn_primary" && action === "click") {
        Vellum UI.ui.setText("status", "Primary button pressed!");
    }

    if (widgetId === "btn_secondary" && action === "click") {
        Vellum UI.ui.setText("status", "Secondary button pressed!");
    }

    if (widgetId === "cb1" && action === "valueChanged") {
        const state = value === 1 ? "ON" : "OFF";
        Vellum UI.ui.setText("status", `Notifications: ${state}`);
    }

    if (widgetId === "cb2" && action === "valueChanged") {
        const state = value === 1 ? "ON" : "OFF";
        Vellum UI.ui.setText("status", `Dark mode: ${state}`);
    }

    if (widgetId === "vol_slider" && action === "valueChanged") {
        const vol = Math.round(value);
        Vellum UI.ui.setText("slider_label", `Volume: ${vol}%`);
        Vellum UI.ui.setText("status", `Slider moved to ${vol}%`);
    }
});

console.log("UI test loaded. Interact with the widgets!");
