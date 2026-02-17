import { createRenderEffect } from "npm:solid-js";
import { createRenderer } from "npm:solid-js/universal";

export type AppJsStyle = Record<string, unknown>;

export interface AppJsEvent {
    type: string;
    widgetId?: string;
    action?: string;
    value?: string | number | boolean;
    width?: number;
    height?: number;
    x?: number;
    y?: number;
    key?: string;
    text?: string;
    focused?: boolean;
}

export interface AppJsRuntime {
    nextId?: () => string;
    ui: {
        createWidget: (
            id: string,
            kind: string,
            parentId: string | null,
            text: string | null,
            style: AppJsStyle | null
        ) => void;
        removeWidget: (id: string) => void;
        setText: (id: string, text: string) => void;
        setVisible: (id: string, visible: boolean) => void;
        setValue: (id: string, value: number) => void;
        setChecked: (id: string, checked: boolean) => void;
        setStyle: (id: string, style: AppJsStyle) => void;
        setStyleProperty: (id: string, property: string, value: string | number | boolean) => void;
    };
    events: {
        on: (type: string, callback: (event: AppJsEvent) => void) => () => void;
    };
}

type WidgetActionHandler = (event: AppJsEvent) => void;

type HostNode = HostElement | HostText;

interface HostCommon {
    parent: HostParent | null;
    firstChild: HostNode | null;
    nextSibling: HostNode | null;
    mounted: boolean;
}

interface HostElement extends HostCommon {
    nodeType: "element";
    tag: string;
    widgetId: string;
    props: Record<string, unknown>;
    handlers: Map<string, Set<WidgetActionHandler>>;
}

interface HostText extends HostCommon {
    nodeType: "text";
    widgetId: string;
    text: string;
}

interface AppJsJsxNode {
    __appjsJsx: true;
    type: string;
    props?: Record<string, unknown>;
}

export interface AppJsRoot {
    nodeType: "root";
    parent: null;
    firstChild: HostNode | null;
    nextSibling: null;
    mounted: true;
    parentWidgetId: string | null;
}

type HostParent = AppJsRoot | HostElement;

export interface RenderOptions {
    parentId?: string | null;
}

export interface AppJsHostElement {
    nodeType: "element";
    widgetId: string;
}

export interface AppJsHostText {
    nodeType: "text";
    widgetId: string;
}

export interface AppJsRenderer {
    createRoot(parentWidgetId?: string | null): AppJsRoot;
    createHostElement(tag: string): AppJsHostElement;
    createHostText(value: string): AppJsHostText;
    setHostProperty(
        node: AppJsHostElement,
        name: string,
        value: unknown,
        prev?: unknown
    ): void;
    appendHostNode(
        parent: AppJsRoot | AppJsHostElement,
        node: AppJsHostElement | AppJsHostText,
        anchor?: AppJsHostElement | AppJsHostText | null
    ): void;
    render(code: () => unknown, options?: RenderOptions | AppJsRoot): AppJsRoot;
    dispose(): void;
}

const DEFAULT_PARENT_ID: string | null = null;
const EVENT_WILDCARD = "widgetAction";

function isEventProp(name: string): boolean {
    return /^on[A-Z]/.test(name);
}

function normalizeEventName(propName: string): string | null {
    if (!isEventProp(propName)) return null;
    const raw = propName.slice(2);
    if (!raw) return null;
    const normalized = raw.charAt(0).toLowerCase() + raw.slice(1);
    return normalized;
}

function normalizeWidgetKind(tag: string): string {
    switch (tag) {
        case "row":
        case "column":
            return "flex";
        case "container":
            return "container";
        case "div":
        case "section":
        case "main":
        case "article":
            return "flex";
        case "span":
        case "p":
        case "h1":
        case "h2":
        case "h3":
        case "h4":
        case "h5":
        case "h6":
            return "label";
        case "input":
            return "textInput";
        case "progress":
            return "progressBar";
        default:
            return tag;
    }
}

function mapStyleKey(name: string): string {
    if (name === "min") return "minValue";
    if (name === "max") return "maxValue";
    if (name === "className") return "class";
    return name;
}

function isNullish(value: unknown): value is null | undefined {
    return value === null || value === undefined;
}

function isPrimitiveStyleValue(value: unknown): value is string | number | boolean {
    return (
        typeof value === "string" ||
        typeof value === "number" ||
        typeof value === "boolean"
    );
}

function unlinkFromParent(parent: HostParent, node: HostNode): void {
    if (parent.firstChild === node) {
        parent.firstChild = node.nextSibling;
        return;
    }

    let current = parent.firstChild;
    while (current && current.nextSibling !== node) {
        current = current.nextSibling;
    }

    if (current) {
        current.nextSibling = node.nextSibling;
    }
}

function linkIntoParent(parent: HostParent, node: HostNode, anchor: HostNode | null): void {
    if (!anchor) {
        node.nextSibling = null;
        if (!parent.firstChild) {
            parent.firstChild = node;
            return;
        }

        let current = parent.firstChild;
        while (current.nextSibling) {
            current = current.nextSibling;
        }

        current.nextSibling = node;
        return;
    }

    if (parent.firstChild === anchor) {
        node.nextSibling = anchor;
        parent.firstChild = node;
        return;
    }

    let current = parent.firstChild;
    while (current && current.nextSibling !== anchor) {
        current = current.nextSibling;
    }

    if (!current) {
        node.nextSibling = null;
        if (!parent.firstChild) {
            parent.firstChild = node;
        } else {
            let tail = parent.firstChild;
            while (tail.nextSibling) {
                tail = tail.nextSibling;
            }
            tail.nextSibling = node;
        }
        return;
    }

    node.nextSibling = anchor;
    current.nextSibling = node;
}

function createEmptyStyle(): AppJsStyle {
    return Object.create(null) as AppJsStyle;
}

function isAppJsJsxNode(value: unknown): value is AppJsJsxNode {
    return (
        typeof value === "object" &&
        value !== null &&
        "__appjsJsx" in value &&
        (value as { __appjsJsx?: unknown }).__appjsJsx === true &&
        typeof (value as { type?: unknown }).type === "string"
    );
}

function isHostNodeLike(value: unknown): value is HostNode {
    return (
        typeof value === "object" &&
        value !== null &&
        "nodeType" in value &&
        (((value as { nodeType?: unknown }).nodeType === "element") ||
            ((value as { nodeType?: unknown }).nodeType === "text"))
    );
}

function normalizeChildrenArray(input: unknown): unknown[] {
    if (Array.isArray(input)) {
        const out: unknown[] = [];
        for (const entry of input) {
            out.push(...normalizeChildrenArray(entry));
        }
        return out;
    }

    if (input === null || input === undefined || input === false || input === true) {
        return [];
    }

    return [input];
}

function isReactiveAccessorProp(name: string, value: unknown): value is () => unknown {
    if (typeof value !== "function") return false;
    if (name === "children" || name === "ref" || name === "key") return false;
    if (isEventProp(name)) return false;
    return true;
}

export function createAppJsRenderer(runtime: AppJsRuntime): AppJsRenderer {
    const widgetNodeById = new Map<string, HostElement>();
    const jsxNodeMap = new WeakMap<object, HostNode>();
    let fallbackId = 0;
    let unsubscribeEvents: (() => void) | null = null;

    function nextWidgetId(prefix: string): string {
        if (runtime.nextId) return runtime.nextId();
        fallbackId += 1;
        return `__solid_${prefix}_${fallbackId}`;
    }

    function ensureEventSubscription(): void {
        if (unsubscribeEvents) return;

        unsubscribeEvents = runtime.events.on(EVENT_WILDCARD, (event) => {
            const widgetId = event.widgetId;
            if (!widgetId) return;

            const node = widgetNodeById.get(widgetId);
            if (!node) return;

            const action = event.action ?? EVENT_WILDCARD;
            const specific = node.handlers.get(action);
            if (specific) {
                for (const handler of specific) handler(event);
            }

            const wildcard = node.handlers.get(EVENT_WILDCARD);
            if (wildcard) {
                for (const handler of wildcard) handler(event);
            }
        });
    }

    function createRoot(parentWidgetId: string | null = DEFAULT_PARENT_ID): AppJsRoot {
        return {
            nodeType: "root",
            parent: null,
            firstChild: null,
            nextSibling: null,
            mounted: true,
            parentWidgetId,
        };
    }

    function getParentWidgetId(parent: HostParent): string | null {
        if (parent.nodeType === "root") return parent.parentWidgetId;
        return parent.widgetId;
    }

    function applyEventProperty(
        node: HostElement,
        propName: string,
        value: unknown,
        prev: unknown
    ): boolean {
        const action = normalizeEventName(propName);
        if (!action) return false;

        const handlers = node.handlers.get(action) ?? new Set<WidgetActionHandler>();

        if (typeof prev === "function") {
            handlers.delete(prev as WidgetActionHandler);
        }

        if (typeof value === "function") {
            handlers.add(value as WidgetActionHandler);
        }

        if (handlers.size > 0) {
            node.handlers.set(action, handlers);
            ensureEventSubscription();
        } else {
            node.handlers.delete(action);
        }

        return true;
    }

    function collectInitialWidgetState(node: HostElement): {
        kind: string;
        text: string | null;
        style: AppJsStyle | null;
    } {
        const kind = normalizeWidgetKind(node.tag);
        const style = createEmptyStyle();
        let hasStyle = false;
        let text: string | null = null;

        if (node.tag === "row") {
            style.direction = "row";
            hasStyle = true;
        } else if (node.tag === "column" || node.tag === "div" || node.tag === "section" || node.tag === "main" || node.tag === "article") {
            style.direction = "column";
            hasStyle = true;
        }

        for (const [name, value] of Object.entries(node.props)) {
            if (name === "children" || name === "ref" || name === "key") continue;
            if (isEventProp(name) || isNullish(value)) continue;
            if (name === "id") continue;
            if (name === "type") continue;
            if (name === "visible") continue;

            if (name === "text") {
                text = String(value);
                continue;
            }

            if (name === "checked") {
                style.checked = Boolean(value);
                hasStyle = true;
                continue;
            }

            if (name === "value" && typeof value === "number") {
                style.progress = value;
                hasStyle = true;
                continue;
            }

            if (name === "style" && typeof value === "object") {
                Object.assign(style, value as AppJsStyle);
                hasStyle = true;
                continue;
            }

            if (isPrimitiveStyleValue(value)) {
                style[mapStyleKey(name)] = value;
                hasStyle = true;
            }
        }

        return {
            kind,
            text,
            style: hasStyle ? style : null,
        };
    }

    function applyMountedProperty(node: HostElement, name: string, value: unknown): void {
        if (name === "children" || name === "ref" || name === "key" || name === "id") return;
        if (name === "type") return;
        if (isEventProp(name)) return;

        if (name === "style") {
            if (value && typeof value === "object") {
                runtime.ui.setStyle(node.widgetId, value as AppJsStyle);
            }
            return;
        }

        if (name === "text") {
            runtime.ui.setText(node.widgetId, String(value ?? ""));
            return;
        }

        if (name === "visible") {
            runtime.ui.setVisible(node.widgetId, Boolean(value));
            return;
        }

        if (name === "checked") {
            runtime.ui.setChecked(node.widgetId, Boolean(value));
            return;
        }

        if (name === "value" && typeof value === "number") {
            runtime.ui.setValue(node.widgetId, value);
            return;
        }

        if (isPrimitiveStyleValue(value)) {
            runtime.ui.setStyleProperty(node.widgetId, mapStyleKey(name), value);
        }
    }

    function mountNode(node: HostNode, parentWidgetId: string | null): void {
        if (node.mounted) return;

        if (node.nodeType === "text") {
            runtime.ui.createWidget(node.widgetId, "label", parentWidgetId, node.text, null);
            node.mounted = true;
            return;
        }

        const init = collectInitialWidgetState(node);
        runtime.ui.createWidget(node.widgetId, init.kind, parentWidgetId, init.text, init.style);
        node.mounted = true;
        widgetNodeById.set(node.widgetId, node);

        for (const [name, value] of Object.entries(node.props)) {
            applyMountedProperty(node, name, value);
        }
    }

    function mountSubtree(node: HostNode, parentWidgetId: string | null): void {
        mountNode(node, parentWidgetId);
        if (node.nodeType === "text") return;

        let child = node.firstChild;
        while (child) {
            mountSubtree(child, node.widgetId);
            child = child.nextSibling;
        }
    }

    function unmountSubtree(node: HostNode): void {
        let child = node.firstChild;
        while (child) {
            const next = child.nextSibling;
            unmountSubtree(child);
            child = next;
        }

        if (node.nodeType === "element") {
            widgetNodeById.delete(node.widgetId);
            node.handlers.clear();
        }

        if (node.mounted) {
            runtime.ui.removeWidget(node.widgetId);
            node.mounted = false;
        }

        node.parent = null;
        node.nextSibling = null;
        node.firstChild = null;
    }

    function buildElementNode(tag: string): HostElement {
        return {
            nodeType: "element",
            tag,
            widgetId: nextWidgetId("el"),
            props: Object.create(null) as Record<string, unknown>,
            handlers: new Map<string, Set<WidgetActionHandler>>(),
            parent: null,
            firstChild: null,
            nextSibling: null,
            mounted: false,
        };
    }

    function buildTextNode(value: string): HostText {
        return {
            nodeType: "text",
            widgetId: nextWidgetId("text"),
            text: String(value),
            parent: null,
            firstChild: null,
            nextSibling: null,
            mounted: false,
        };
    }

    function setElementProperty(node: HostElement, name: string, value: unknown, prev: unknown): void {
        if (name === "ref" && typeof value === "function") {
            value(node);
            return;
        }

        if (name === "id" && typeof value === "string" && !node.mounted) {
            node.widgetId = value;
        }

        const hadProp = Object.prototype.hasOwnProperty.call(node.props, name);
        if (isNullish(value) || value === false) {
            if (hadProp) {
                delete node.props[name];
            }
        } else {
            node.props[name] = value;
        }

        if (applyEventProperty(node, name, value, prev)) {
            return;
        }

        if (node.mounted) {
            applyMountedProperty(node, name, value);
        }
    }

    function insertHostNode(parent: HostParent, node: HostNode, anchor: HostNode | null = null): void {
        node.parent = parent;
        linkIntoParent(parent, node, anchor);

        if (parent.mounted) {
            mountSubtree(node, getParentWidgetId(parent));
        }
    }

    function clearElementChildren(element: HostElement): void {
        let child = element.firstChild;
        while (child) {
            const next = child.nextSibling;
            unmountSubtree(child);
            child = next;
        }
        element.firstChild = null;
    }

    function reconcileElementChildren(element: HostElement, childrenValue: unknown): void {
        clearElementChildren(element);

        const children = normalizeChildrenArray(childrenValue);
        for (const child of children) {
            const hostChild = materializeHostNode(child);
            if (!hostChild) continue;
            insertHostNode(element, hostChild, null);
        }
    }

    function materializeHostNode(input: unknown): HostNode | null {
        if (isHostNodeLike(input)) {
            return input;
        }

        if (typeof input === "string" || typeof input === "number") {
            return buildTextNode(String(input));
        }

        if (!isAppJsJsxNode(input)) {
            return null;
        }

        const cached = jsxNodeMap.get(input as object);
        if (cached) {
            return cached;
        }

        const element = buildElementNode(input.type);
        jsxNodeMap.set(input as object, element);

        const props = input.props ?? {};
        for (const [key, value] of Object.entries(props)) {
            if (key === "children") continue;

            if (isReactiveAccessorProp(key, value)) {
                let prev: unknown = undefined;
                createRenderEffect(() => {
                    const next = value();
                    setElementProperty(element, key, next, prev);
                    prev = next;
                });
                continue;
            }

            setElementProperty(element, key, value, undefined);
        }

        const childrenProp = props.children;
        if (typeof childrenProp === "function") {
            createRenderEffect(() => {
                reconcileElementChildren(element, childrenProp());
            });
        } else {
            reconcileElementChildren(element, childrenProp);
        }

        return element;
    }

    const renderer = createRenderer<HostNode | AppJsRoot>({
        createElement(tag: string): HostElement {
            return buildElementNode(tag);
        },
        createTextNode(value: string): HostText {
            return buildTextNode(value);
        },
        replaceText(node: HostNode | AppJsRoot, value: string): void {
            if (node.nodeType !== "text") return;

            node.text = String(value);
            if (node.mounted) {
                runtime.ui.setText(node.widgetId, node.text);
            }
        },
        setProperty(node: HostNode | AppJsRoot, name: string, value: unknown, prev: unknown): void {
            if (node.nodeType !== "element") return;
            setElementProperty(node, name, value, prev);
        },
        insertNode(parent: HostNode | AppJsRoot, node: HostNode | AppJsRoot, anchor?: HostNode | AppJsRoot): void {
            const hostParent = parent as HostParent;
            const hostNode = materializeHostNode(node);
            if (!hostNode) return;

            const hostAnchor = materializeHostNode(anchor) ?? null;
            insertHostNode(hostParent, hostNode, hostAnchor);
        },
        isTextNode(node: HostNode | AppJsRoot): boolean {
            return node.nodeType === "text";
        },
        removeNode(parent: HostNode | AppJsRoot, node: HostNode | AppJsRoot): void {
            const hostParent = parent as HostParent;
            const hostNode = materializeHostNode(node);
            if (!hostNode) return;

            unlinkFromParent(hostParent, hostNode);
            unmountSubtree(hostNode);
        },
        getParentNode(node: HostNode | AppJsRoot): HostParent | null {
            return node.parent;
        },
        getFirstChild(node: HostNode | AppJsRoot): HostNode | null {
            return node.firstChild;
        },
        getNextSibling(node: HostNode | AppJsRoot): HostNode | null {
            return node.nextSibling;
        },
    });

    function render(code: () => unknown, options?: RenderOptions | AppJsRoot): AppJsRoot {
        const root =
            options && "nodeType" in options
                ? options
                : createRoot(options?.parentId ?? DEFAULT_PARENT_ID);

        renderer.render(code, root);
        return root;
    }

    function dispose(): void {
        if (unsubscribeEvents) {
            unsubscribeEvents();
            unsubscribeEvents = null;
        }
        widgetNodeById.clear();
    }

    return {
        ...renderer,
        createRoot,
        createHostElement: (tag: string): AppJsHostElement => buildElementNode(tag),
        createHostText: (value: string): AppJsHostText => buildTextNode(value),
        setHostProperty: (node: AppJsHostElement, name: string, value: unknown, prev?: unknown): void => {
            setElementProperty(node as HostElement, name, value, prev);
        },
        appendHostNode: (
            parent: AppJsRoot | AppJsHostElement,
            node: AppJsHostElement | AppJsHostText,
            anchor?: AppJsHostElement | AppJsHostText | null
        ): void => {
            insertHostNode(parent as HostParent, node as HostNode, (anchor as HostNode | null | undefined) ?? null);
        },
        render,
        dispose,
    };
}

export interface AppJsCommonProps {
    // Accessor form is supported for non-event props by renderer reactivity.
    // Keep this broad to avoid fighting editor JSX inference.
    id?: string;
    key?: string | number;
    ref?: (node: unknown) => void;
    style?: AppJsStyle | (() => AppJsStyle);
    text?: string | (() => string);
    value?: number | (() => number);
    checked?: boolean | (() => boolean);
    visible?: boolean | (() => boolean);
    onClick?: WidgetActionHandler;
    onValueChanged?: WidgetActionHandler;
    onTextChanged?: WidgetActionHandler;
    onWidgetAction?: WidgetActionHandler;
    [key: string]: unknown;
}

export type AppJsIntrinsicElements = {
    [tagName: string]: AppJsCommonProps;
    label: AppJsCommonProps;
    button: AppJsCommonProps;
    checkbox: AppJsCommonProps;
    textInput: AppJsCommonProps;
    slider: AppJsCommonProps;
    progressBar: AppJsCommonProps;
    spinner: AppJsCommonProps;
    prose: AppJsCommonProps;
    flex: AppJsCommonProps;
    row: AppJsCommonProps;
    column: AppJsCommonProps;
    box: AppJsCommonProps;
    zstack: AppJsCommonProps;
    portal: AppJsCommonProps;
};
