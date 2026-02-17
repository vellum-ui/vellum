// Minimal test: just create a label with text
import * as appjs from "../packages/appjs-runtime/src/index.ts";

appjs.window.setTitle("Simple Test");
appjs.ui.createWidget("lbl", "Label");
appjs.ui.setWidgetText("lbl", "Hello World - This should be visible!");
console.log("Done - label created with text");
