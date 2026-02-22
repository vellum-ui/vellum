// Styled Counter App Example — demonstrates the new styling API
import * as Vellum UI from "@vellum/core";

Vellum UI.window.setTitle("Styled Counter");

// Root container fills the window with dark background and centers content
Vellum UI.column("main_col", null, {
    flex: 1,
    gap: 16,
    padding: 24,
    background: "#1e1e2e",
    crossAxisAlignment: "center",
});

// Header label with large font
Vellum UI.label("header", "main_col", "✨ Styled Counter", {
    fontSize: 28,
    fontWeight: 700,
    color: "#cdd6f4",
});

// Counter display
Vellum UI.label("countLabel", "main_col", "Count: 0", {
    fontSize: 48,
    fontWeight: 900,
    color: "#f38ba8",
});

// Button row
const btnRow = Vellum UI.row("btn_row", "main_col", {
    gap: 12,
    mainAxisAlignment: "center",
});

Vellum UI.button("decBtn", "btn_row", "  −  ");
Vellum UI.button("incBtn", "btn_row", "  +  ");

// A progress bar that tracks count 0-10
Vellum UI.label("progressLabel", "main_col", "Progress (0-10):", {
    fontSize: 14,
    color: "#a6adc8",
});
Vellum UI.progressBar("prog", "main_col", 0.0);

// A slider
Vellum UI.label("sliderLabel", "main_col", "Slider:", {
    fontSize: 14,
    color: "#a6adc8",
});
Vellum UI.slider("mySlider", "main_col", 0.0, 100.0, 50.0);

// Checkbox
Vellum UI.checkbox("darkMode", "main_col", true, "Dark Mode");

let count = 0;

function updateCount() {
    Vellum UI.ui.setText("countLabel", `Count: ${count}`);
    const progress = Math.max(0, Math.min(1, count / 10));
    Vellum UI.ui.setValue("prog", progress);

    // Update label color based on count
    if (count > 0) {
        Vellum UI.ui.setStyleProperty("countLabel", "color", "#a6e3a1");
    } else if (count < 0) {
        Vellum UI.ui.setStyleProperty("countLabel", "color", "#f38ba8");
    } else {
        Vellum UI.ui.setStyleProperty("countLabel", "color", "#cdd6f4");
    }
}

Vellum UI.events.on("widgetAction", (e) => {
    if (e.widgetId === "incBtn") {
        count++;
        updateCount();
    } else if (e.widgetId === "decBtn") {
        count--;
        updateCount();
    }
});

console.info("Styled counter app initialized.");
