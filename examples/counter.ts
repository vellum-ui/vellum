// Counter App Example — TypeScript Version
// Demonstrates that .ts files work with full type annotations
/// <reference path="../types/appjs.d.ts" />

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
    appjs.ui.setText("countLabel", `Count: ${s.count}`);
    appjs.ui.setText("stepLabel", `Step: ${s.step}`);
}

// Set up the UI
appjs.window.setTitle("TypeScript Counter");

appjs.body.setStyle({ background: "#1e1e2e", padding: 24 });

appjs.column("root", null, {
    gap: 16,
    crossAxisAlignment: "center",
});

appjs.label("header", "root", "TypeScript Counter", {
    fontSize: 28,
    fontWeight: 700,
    color: "#cdd6f4",
});

appjs.label("countLabel", "root", `Count: ${state.count}`, {
    fontSize: 48,
    fontWeight: 900,
    color: "#89b4fa",
});

appjs.label("stepLabel", "root", `Step: ${state.step}`, {
    fontSize: 16,
    color: "#a6adc8",
});

// Buttons row
appjs.row("btnRow", "root", { gap: 12 });
appjs.button("decBtn", "btnRow", "−");
appjs.button("resetBtn", "btnRow", "Reset");
appjs.button("incBtn", "btnRow", "⁺");

// Step control
appjs.row("stepRow", "root", { gap: 12 });
appjs.label("stepTitle", "stepRow", "Step size:", { color: "#a6adc8" });
appjs.button("step1", "stepRow", "1");
appjs.button("step5", "stepRow", "5");
appjs.button("step10", "stepRow", "10");

// Event handling with typed events
appjs.events.on("widgetAction", (e: AppJsEvent) => {
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

appjs.log.info("TypeScript counter app initialized!");
