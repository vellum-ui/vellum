import * as Vellum from "@vellum-ui/core";
import { createVellumRenderer, createSignal } from "@vellum-ui/solid";
import { Minus, RotateCcw, Plus } from "lucide-static";

Vellum.window.setTitle("Vellum Solid TSX Demo");
Vellum.window.resize(700, 460);
Vellum.body.setStyle({
    background: "#1e1e2e",
    padding: 24,
});

const renderer = createVellumRenderer(Vellum);

function Counter() {
    const [count, setCount] = createSignal(0);

    return (
        <column gap={14} crossAxisAlignment="center">
            <label
                text="Solid TSX + Vellum Custom Renderer"
                fontSize={24}
                fontWeight={700}
                color="#cdd6f4"
            />

            <label
                text={() => `Count: ${count()}`}
                fontSize={56}
                fontWeight={900}
                style={() => ({
                    color: count() >= 0 ? "#89b4fa" : "#f38ba8",
                    fontSize: 56,
                    fontWeight: 900,
                })}
            />

            <row gap={10}>
                <button onClick={() => setCount((value: number) => value - 1)}>
                    <svg svg_data={Minus} color="white" />
                </button>
                <button onClick={() => setCount(0)} direction="row" gap={6}>
                    <label text="Reset" color="white" />
                    <svg svg_data={RotateCcw} color="white" />
                </button>
                <button onClick={() => setCount((value: number) => value + 1)}>
                    <svg svg_data={Plus} color="white" />
                </button>
            </row>

            <label
                text="This screen is authored as TSX and transpiled by Vellum module loader."
                fontSize={13}
                color="#a6adc8"
            />

            <label
                text={() => (count() % 2 === 0 ? "Count is even" : "Count is odd")}
                fontSize={12}
                color={() => (count() % 2 === 0 ? "#a6e3a1" : "#f9e2af")}
            />
        </column>
    );
}

renderer.render(() => <Counter />);
console.info("Solid TSX demo initialized");
