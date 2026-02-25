/** @jsxImportSource @vellum-ui/solid */
import * as Vellum from "@vellum-ui/core";
import { createVellumRenderer, createSignal, onMount } from "@vellum-ui/solid";

Vellum.window.setTitle("Vellum Video Player");
Vellum.window.resize(800, 600);
Vellum.body.setStyle({
  backgroundColor: "#111111",
  mustFillMainAxis: true,
});

const renderer = createVellumRenderer(Vellum);

const VIDEO_ID = "main-video-player";

function VideoPlayer() {
  const [isPlaying, setIsPlaying] = createSignal(true);
  const [videoUrl, setVideoUrl] = createSignal("");
  const [videoPosition, setVideoPosition] = createSignal<number | undefined>(undefined);

  onMount(() => {
    // We can use a test video URL or a local file
    setVideoUrl("file:///C:/Users/karma/Documents/Repos/vellum/.ref/big-buck-bunny-1080p-60fps-30sec.mp4");
    // setVideoUrl("https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4");
  });

  const togglePlay = () => {
    setIsPlaying(!isPlaying());
  };

  const seekToStart = () => {
    setVideoPosition(0);
  };

  const seekForward = () => {
    setVideoPosition(10);
  };

  return (
    <column style={{ padding: 20, gap: 20, width: "100%", height: "100%", crossAxisAlignment: "center" }}>
      <label text="Vellum Video Player" style={{ fontSize: 24, fontWeight: 700, color: "#FFFFFF" }} />

      {videoUrl() ? (
        <video
          id={VIDEO_ID}
          src={videoUrl()}
          playing={() => isPlaying()}
          position={() => videoPosition() || 0}
          style={{ width: 640, height: 360, backgroundColor: "#000000", cornerRadius: 8 }}
        />
      ) : (
        <box style={{ width: 640, height: 360, backgroundColor: "#222222", cornerRadius: 8, crossAxisAlignment: "center", mainAxisAlignment: "center" }}>
          <label text="Loading..." style={{ color: "#888888" }} />
        </box>
      )}

      <row style={{ gap: 10 }}>
        <button
          onClick={togglePlay}
          style={{
            padding: { top: 10, bottom: 10, left: 20, right: 20 },
            backgroundColor: "#3b82f6",
            cornerRadius: 6,
          }}
        >
          <label text={() => isPlaying() ? "Pause" : "Play"} style={{ color: "#ffffff" }} />
        </button>
        <button
          onClick={seekToStart}
          style={{
            padding: { top: 10, bottom: 10, left: 20, right: 20 },
            backgroundColor: "#4b5563",
            cornerRadius: 6,
          }}
        >
          <label text="Restart" style={{ color: "#ffffff" }} />
        </button>
        <button
          onClick={seekForward}
          style={{
            padding: { top: 10, bottom: 10, left: 20, right: 20 },
            backgroundColor: "#4b5563",
            cornerRadius: 6,
          }}
        >
          <label text="Seek to 10s" style={{ color: "#ffffff" }} />
        </button>
      </row>
    </column>
  );
}

renderer.render(() => <VideoPlayer />);
