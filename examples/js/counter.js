// Counter App Example
import * as Vellum UI from "@vellum/core";

// Set window title
Vellum UI.window.setTitle("Counter App");

// Create a root column that fills the window
Vellum UI.column("root", null, {
    gap: 12,
    padding: 20,
    crossAxisAlignment: "center",
});

Vellum UI.label("header", "root", "Simple Counter App", { fontSize: 24, fontWeight: 700 });
Vellum UI.label("countLabel", "root", "Count: 0", { fontSize: 36, fontWeight: 900 });

Vellum UI.row("btnRow", "root", { gap: 10 });
Vellum UI.button("incBtn", "btnRow", "Increment");
Vellum UI.button("decBtn", "btnRow", "Decrement");

let count = 0;

function updateCount() {
    Vellum UI.ui.setWidgetText("countLabel", `Count: ${count}`);
}

Vellum UI.events.on("widgetAction", (e) => {
    // Log the event for debugging
    console.info(`Action on ${e.widgetId}: ${e.action}`);

    if (e.widgetId === "incBtn") {
        count++;
        updateCount();
    } else if (e.widgetId === "decBtn") {
        count--;
        updateCount();
    }
});

console.info("Counter app initialized. Waiting for clicks...");
