# @vellum/solid

SolidJS custom renderer targeting the Vellum UI runtime bridge.

## What Works Today

- TSX rendering with `renderer.render(() => <App />)`.
- Reactive accessor props for common fields (`text`, `style`, `color`, etc.).
- Event handlers via widget actions (`onClick`, `onValueChanged`,
  `onTextChanged`, `onWidgetAction`).
- Declarative state-driven UI with `createSignal` and Solid effects.

## Runtime Imports

In external Vellum UI projects (examples repo), use published npm specifiers:

```ts
import { createSignal } from "npm:solid-js";
import * as Vellum UI from "npm:@vellum/core";
import { createVellum UIRenderer } from "npm:@vellum/solid";
```

## Usage (Idiomatic TSX)

```tsx
import { createSignal } from "npm:solid-js";
import * as Vellum UI from "npm:@vellum/core";
import { createVellum UIRenderer } from "npm:@vellum/solid";

const renderer = createVellum UIRenderer(Vellum UI);

function Counter() {
    const [count, setCount] = createSignal(0);

    return (
        <column gap={12} crossAxisAlignment="center">
            <label text={() => `Count: ${count()}`} fontSize={28} />

            <row gap={8}>
                <button
                    text="-"
                    onClick={() => setCount((value: number) => value - 1)}
                />
                <button text="Reset" onClick={() => setCount(0)} />
                <button
                    text="+"
                    onClick={() => setCount((value: number) => value + 1)}
                />
            </row>

            <label
                text={() => (count() % 2 === 0
                    ? "Count is even"
                    : "Count is odd")}
                color={() => (count() % 2 === 0 ? "#a6e3a1" : "#f9e2af")}
            />
        </column>
    );
}

renderer.render(() => <Counter />);
```

## Accessor Props

Function-valued non-event props are tracked reactively:

- `text={() => ...}`
- `style={() => ({ ... })}`
- `color={() => ...}` and other primitive style props
- `children={() => ...}` (reconciled as dynamic children)

## Current Limitations

- Dynamic `children` updates currently use full subtree replacement for that
  node (no keyed diff yet).
- Event surface is currently based on Vellum UI `widgetAction` events.
- Dev-time editor diagnostics in VS Code may not fully resolve `npm:`
  specifiers, but runtime execution works.

## Legacy Imperative API

`createHostElement`, `appendHostNode`, and `setHostProperty` remain available
for low-level/manual flows.

## Install

```bash
npm install @vellum/core @vellum/solid solid-js
```

## Tag Mapping

- Intrinsic Vellum UI tags: `column`, `row`, `button`, `label`, `checkbox`,
  `textInput`, `slider`, etc.
- HTML-like aliases: `div`/`section`/`main`/`article` -> `flex` with column
  direction.
- Text aliases: `span`/`p`/`h1..h6` -> `label`.
