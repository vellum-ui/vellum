import * as appjs from "@appjs/runtime";
import { createAppJsRenderer, createSignal } from "@appjs/solid-renderer";

appjs.window.setTitle("AppJS Hover Test");
appjs.window.resize(760, 520);
appjs.body.setStyle({
  background: "#111827",
  padding: 24,
});

const renderer = createAppJsRenderer(appjs);

function HoverExample() {
  const [buttonHovered, setButtonHovered] = createSignal(false);
  const [cardHovered, setCardHovered] = createSignal(false);

  return (
    <column gap={16} crossAxisAlignment="start">
      <label
        text="Hover wrapper test"
        fontSize={28}
        fontWeight={800}
        color="#f9fafb"
      />

      <label
        text="Move the pointer over the card and button. Styles update from onHover events."
        fontSize={14}
        color="#9ca3af"
      />

      <column
        width={680}
        padding="16,16,16,16"
        cornerRadius={16}
        borderWidth={2}
        style={() => ({
          background: cardHovered() ? "#1f2937" : "#0f172a",
          borderColor: cardHovered() ? "#60a5fa" : "#334155",
        })}
        onHover={(event) => {
          setCardHovered(Boolean(event.value));
        }}
      >
        <label
          text={() =>
            cardHovered() ? "Card hover: ENTERED" : "Card hover: LEFT"
          }
          fontSize={16}
          fontWeight={700}
          color={() => (cardHovered() ? "#93c5fd" : "#cbd5e1")}
        />

        <row gap={10} style={{ marginTop: 8 }}>
          <button
            onHover={(event) => {
              setButtonHovered(Boolean(event.value));
            }}
            style={() => ({
              background: buttonHovered() ? "#2563eb" : "#374151",
              borderColor: buttonHovered() ? "#93c5fd" : "#4b5563",
              borderWidth: 2,
              cornerRadius: 10,
              padding: "10,14,10,14",
            })}
          >
            <label
              text={() => (buttonHovered() ? "Hovered" : "Hover me")}
              color="#ffffff"
              fontWeight={700}
            />
          </button>

          <label
            text={() =>
              buttonHovered()
                ? "Button hover: ENTERED"
                : "Button hover: LEFT"
            }
            fontSize={14}
            color={() => (buttonHovered() ? "#bfdbfe" : "#9ca3af")}
          />
        </row>
      </column>
    </column>
  );
}

renderer.render(() => <HoverExample />);
console.info("Hover example initialized");
