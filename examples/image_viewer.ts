// Image Viewer Example
// Fetches a random image from picsum.photos, shows a spinner while loading,
// and provides a button to refetch a new image.
import * as Vellum from "@vellum/core";
import type { VellumEvent } from "@vellum/core";

const WIDTH = 400;
const HEIGHT = 300;

Vellum.window.setTitle("Image Viewer");
Vellum.body.setStyle({ background: "#1e1e2e", padding: 24 });

Vellum.column("root", null, {
  gap: 16,
  crossAxisAlignment: "center",
});

Vellum.label("header", "root", "Random Image Viewer", {
  fontSize: 28,
  fontWeight: 700,
  color: "#cdd6f4",
});

Vellum.label("status", "root", "Loading...", {
  fontSize: 14,
  color: "#a6adc8",
});

// Container for the image / spinner area (must be Flex so removeWidget works)
Vellum.column("imageArea", "root", {
  width: WIDTH,
  height: HEIGHT,
  crossAxisAlignment: "center",
});

Vellum.button("fetchBtn", "root", "🔄  New Image");

let imageCreated = false;
let spinnerShown = false;

function showSpinner(): void {
  if (!spinnerShown) {
    Vellum.spinner("loadingSpinner", "imageArea");
    spinnerShown = true;
  }
}

function hideSpinner(): void {
  if (spinnerShown) {
    Vellum.ui.removeWidget("loadingSpinner");
    spinnerShown = false;
  }
}

// Show spinner initially
showSpinner();

async function fetchImage(): Promise<void> {
  showSpinner();
  Vellum.ui.setText("status", "Fetching image...");

  try {
    // picsum.photos redirects to a random image each time
    const url = `https://picsum.photos/${WIDTH}/${HEIGHT}?t=${Date.now()}`;
    const response = await fetch(url, { redirect: "follow" });
    const buffer = await response.arrayBuffer();
    const data = new Uint8Array(buffer);

    hideSpinner();

    if (!imageCreated) {
      Vellum.image("img", "imageArea", data, {
        objectFit: "contain",
        width: WIDTH,
        height: HEIGHT,
      });
      imageCreated = true;
    } else {
      Vellum.ui.setImageData("img", data);
    }

    Vellum.ui.setText("status", `Image loaded (${data.byteLength} bytes)`);
  } catch (err) {
    hideSpinner();
    Vellum.ui.setText("status", `Error: ${err}`);
  }
}

Vellum.events.on("widgetAction", (e: VellumEvent) => {
  if (e.widgetId === "fetchBtn") {
    fetchImage();
  }
});

// Fetch the first image on startup
fetchImage();

console.info("Image viewer app initialized!");
