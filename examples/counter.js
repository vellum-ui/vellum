// Counter App Example

// Set window title
appjs.window.setTitle("Counter App");

// Create a root column that fills the window
appjs.column("root", null, {
    gap: 12,
    padding: 20,
    crossAxisAlignment: "center",
});

appjs.label("header", "root", "Simple Counter App", { fontSize: 24, fontWeight: 700 });
appjs.label("countLabel", "root", "Count: 0", { fontSize: 36, fontWeight: 900 });

appjs.row("btnRow", "root", { gap: 10 });
appjs.button("incBtn", "btnRow", "Increment");
appjs.button("decBtn", "btnRow", "Decrement");

let count = 0;

function updateCount() {
    appjs.ui.setWidgetText("countLabel", `Count: ${count}`);
}

appjs.events.on("widgetAction", (e) => {
    // Log the event for debugging
    appjs.log.info(`Action on ${e.widgetId}: ${e.action}`);

    if (e.widgetId === "incBtn") {
        count++;
        updateCount();
    } else if (e.widgetId === "decBtn") {
        count--;
        updateCount();
    }
});

appjs.log.info("Counter app initialized. Waiting for clicks...");
