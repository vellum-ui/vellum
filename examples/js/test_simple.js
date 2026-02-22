// Minimal test: just create a label with text
import * as Vellum UI from "@vellum/core";

Vellum UI.window.setTitle("Simple Test");
Vellum UI.ui.createWidget("lbl", "Label");
Vellum UI.ui.setWidgetText("lbl", "Hello World - This should be visible!");
console.log("Done - label created with text");
