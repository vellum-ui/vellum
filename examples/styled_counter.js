// Styled Counter App Example — demonstrates the new styling API

appjs.window.setTitle("Styled Counter");

// Root container fills the window with dark background and centers content
appjs.column("main_col", null, {
    flex: 1,
    gap: 16,
    padding: 24,
    background: "#1e1e2e",
    crossAxisAlignment: "center",
});

// Header label with large font
appjs.label("header", "main_col", "✨ Styled Counter", {
    fontSize: 28,
    fontWeight: 700,
    color: "#cdd6f4",
});

// Counter display
appjs.label("countLabel", "main_col", "Count: 0", {
    fontSize: 48,
    fontWeight: 900,
    color: "#f38ba8",
});

// Button row
const btnRow = appjs.row("btn_row", "main_col", {
    gap: 12,
    mainAxisAlignment: "center",
});

appjs.button("decBtn", "btn_row", "  −  ");
appjs.button("incBtn", "btn_row", "  +  ");

// A progress bar that tracks count 0-10
appjs.label("progressLabel", "main_col", "Progress (0-10):", {
    fontSize: 14,
    color: "#a6adc8",
});
appjs.progressBar("prog", "main_col", 0.0);

// A slider
appjs.label("sliderLabel", "main_col", "Slider:", {
    fontSize: 14,
    color: "#a6adc8",
});
appjs.slider("mySlider", "main_col", 0.0, 100.0, 50.0);

// Checkbox
appjs.checkbox("darkMode", "main_col", true, "Dark Mode");

let count = 0;

function updateCount() {
    appjs.ui.setText("countLabel", `Count: ${count}`);
    const progress = Math.max(0, Math.min(1, count / 10));
    appjs.ui.setValue("prog", progress);

    // Update label color based on count
    if (count > 0) {
        appjs.ui.setStyleProperty("countLabel", "color", "#a6e3a1");
    } else if (count < 0) {
        appjs.ui.setStyleProperty("countLabel", "color", "#f38ba8");
    } else {
        appjs.ui.setStyleProperty("countLabel", "color", "#cdd6f4");
    }
}

appjs.events.on("widgetAction", (e) => {
    if (e.widgetId === "incBtn") {
        count++;
        updateCount();
    } else if (e.widgetId === "decBtn") {
        count--;
        updateCount();
    }
});

appjs.log.info("Styled counter app initialized.");
