// Counter App Example — TypeScript Version
// Demonstrates that .ts files work with full type annotations
import { body, button, column, events, label, row, ui, window } from "@vellum/core";
import type { VellumEvent } from "@vellum/core";

// ---- App logic with TypeScript features ----

interface CounterState {
    count: number;
    step: number;
}

const state: CounterState = {
    count: 0,
    step: 1,
};

function updateDisplay(s: CounterState): void {
    ui.setText("countLabel", `Count: ${s.count}`);
    ui.setText("stepLabel", `Step: ${s.step}`);
}

// Set up the UI
window.setTitle("TypeScript Counter");

body.setStyle({ background: "#1e1e2e", padding: 24 });

column("root", null, {
    gap: 16,
    crossAxisAlignment: "center",
});

label("header", "root", "TypeScript Counter", {
    fontSize: 28,
    fontWeight: 700,
    color: "#cdd6f4",
});

label("countLabel", "root", `Count: ${state.count}`, {
    fontSize: 48,
    fontWeight: 900,
    color: "#89b4fa",
});

label("stepLabel", "root", `Step: ${state.step}`, {
    fontSize: 16,
    color: "#a6adc8",
});

// Buttons row
row("btnRow", "root", { gap: 12 });
button("decBtn", "btnRow", "−");
button("resetBtn", "btnRow", "Reset");
button("incBtn", "btnRow", "⁺");

// Step control
row("stepRow", "root", { gap: 12 });
label("stepTitle", "stepRow", "Step size:", { color: "#a6adc8" });
button("step1", "stepRow", "1");
button("step5", "stepRow", "5");
button("step10", "stepRow", "10");

// Event handling with typed events
events.on("widgetAction", (e: VellumEvent) => {
    switch (e.widgetId) {
        case "incBtn":
            state.count += state.step;
            break;
        case "decBtn":
            state.count -= state.step;
            break;
        case "resetBtn":
            state.count = 0;
            break;
        case "step1":
            state.step = 1;
            break;
        case "step5":
            state.step = 5;
            break;
        case "step10":
            state.step = 10;
            break;
    }
    updateDisplay(state);
});

console.info("TypeScript counter app initialized!");
