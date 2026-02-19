import * as appjs from "@appjs/runtime";
import { createAppJsRenderer, createSignal } from "@appjs/solid-renderer";

appjs.window.setTitle("AppJS Solid TSX Demo");
appjs.window.resize(700, 460);
appjs.body.setStyle({
    background: "#1e1e2e",
    padding: 24,
});

const renderer = createAppJsRenderer(appjs);

function Counter() {
    const [count, setCount] = createSignal(0);

    return (
        <column gap={14} crossAxisAlignment="center">
            <label
                text="Solid TSX + AppJS Custom Renderer"
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
                <button type="button" text="-" onClick={() => setCount((value: number) => value - 1)} />
                <button type="button" text="Reset" onClick={() => setCount(0)} />
                <button type="button" text="+" onClick={() => setCount((value: number) => value + 1)} />
            </row>

            <label
                text="This screen is authored as TSX and transpiled by appjs module loader."
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
appjs.log.info("Solid TSX demo initialized");
