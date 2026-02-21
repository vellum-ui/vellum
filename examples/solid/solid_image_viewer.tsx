// Image Viewer — Solid TSX version
// Fetches random images from picsum.photos
import * as appjs from "@appjs/runtime";
import { createAppJsRenderer, createSignal } from "@appjs/solid-renderer";

const WIDTH = 400;
const HEIGHT = 300;

async function fetchImageBytes(): Promise<Uint8Array> {
  const url = `https://picsum.photos/${WIDTH}/${HEIGHT}?t=${Date.now()}`;
  const response = await fetch(url, { redirect: "follow" });
  return new Uint8Array(await response.arrayBuffer());
}

appjs.window.setTitle("Image Viewer");
appjs.window.resize(500, 500);
appjs.body.setStyle({ background: "#1e1e2e", padding: 24 });

const renderer = createAppJsRenderer(appjs);

function ImageViewer() {
  const [status, setStatus] = createSignal("Idle");
  const [isLoading, setIsLoading] = createSignal(false);
  const [imageData, setImageData] = createSignal<Uint8Array | null>(null);

  async function refetch() {
    setIsLoading(true);
    setStatus("Fetching image...");
    try {
      const data = await fetchImageBytes();
      setImageData(data);
      setStatus(`Loaded (${data.byteLength} bytes)`);
    } catch (err) {
      setStatus(`Error: ${err}`);
    } finally {
      setIsLoading(false);
    }
  }

  void refetch();

  return (
    <column gap={16} crossAxisAlignment="center">
      <label
        text="Random Image Viewer"
        fontSize={28}
        fontWeight={700}
        color="#cdd6f4"
      />
      <label
        text={() => status()}
        fontSize={14}
        color="#a6adc8"
      />
      <column width={WIDTH} height={HEIGHT} crossAxisAlignment="center" mainAxisAlignment="center">
        {() => {
          if (isLoading()) {
            return (
              <box width={20} height={20}>
                <spinner />
              </box>
            );
          }

          const data = imageData();
          if (data) {
            return (
              <image
                id="img"
                data={data}
                objectFit="contain"
                width={WIDTH}
                height={HEIGHT}
              />
            );
          }

          return null;
        }}
      </column>
      <button onClick={() => refetch()}>
        <label text="🔄  New Image" color="white" />
      </button>
    </column>
  );
}

renderer.render(() => <ImageViewer />);
console.info("Solid image viewer initialized");
