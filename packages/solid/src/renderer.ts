
import { createRenderEffect, runWithOwner } from "solid-js/dist/solid.js";
import type { Owner } from "solid-js";
import { createRenderer } from "solid-js/universal";
import {
  VellumRenderer,
  VellumRuntime,
  VellumRoot,
  VellumHostElement,
  VellumHostText,
  HostNode,
  HostElement,
  HostText,
  HostParent,
  RenderOptions,
  WidgetActionHandler,
  VellumStyle,
} from "./types";
import {
  DEFAULT_PARENT_ID,
  EVENT_WILDCARD,
} from "./constants";
import {
  isEventProp,
  normalizeEventName,
  normalizeWidgetKind,
  mapStyleKey,
  isNullish,
  isPrimitiveStyleValue,
  createEmptyStyle,
  isVellumJsxNode,
  isHostNodeLike,
  normalizeChildrenArray,
  isReactiveAccessorProp,
  unlinkFromParent,
  linkIntoParent,
  resolveChildValue,
} from "./utils";

export function createVellumRenderer(runtime: VellumRuntime): VellumRenderer {
  const widgetNodeById = new Map<string, HostElement>();
  const jsxNodeMap = new WeakMap<object, HostNode>();
  let fallbackId = 0;
  let unsubscribeEvents: (() => void) | null = null;

  function hasDynamicChildren(input: unknown): boolean {
    if (typeof input === "function") return true;
    if (!Array.isArray(input)) return false;
    for (const entry of input) {
      if (hasDynamicChildren(entry)) return true;
    }
    return false;
  }

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

  function createRoot(parentWidgetId: string | null = DEFAULT_PARENT_ID): VellumRoot {
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
    style: VellumStyle | null;
    params: Record<string, unknown> | null;
    data: Uint8Array | null;
  } {
    const kind = normalizeWidgetKind(node.tag);
    const style = createEmptyStyle();
    const params: Record<string, unknown> = Object.create(null);
    let hasStyle = false;
    let hasParams = false;
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
      if (name === "data") continue;
      if (name === "objectFit") continue;

      if (name === "text") {
        text = String(value);
        continue;
      }

      if (name === "checked") {
        if (kind === "checkbox") {
          params.checked = Boolean(value);
          hasParams = true;
        }
        continue;
      }

      if (name === "value" && typeof value === "number") {
        if (kind === "slider" || kind === "progressBar") {
          params.value = value;
          hasParams = true;
        }
        continue;
      }

      if (name === "min" && typeof value === "number" && kind === "slider") {
        params.minValue = value;
        hasParams = true;
        continue;
      }

      if (name === "max" && typeof value === "number" && kind === "slider") {
        params.maxValue = value;
        hasParams = true;
        continue;
      }

      if (name === "step" && typeof value === "number" && kind === "slider") {
        params.step = value;
        hasParams = true;
        continue;
      }

      if (name === "placeholder" && typeof value === "string" && kind === "textInput") {
        params.placeholder = value;
        hasParams = true;
        continue;
      }

      if (name === "style" && typeof value === "object") {
        Object.assign(style, value as VellumStyle);
        hasStyle = true;
        continue;
      }

      if (isPrimitiveStyleValue(value)) {
        style[mapStyleKey(name)] = value;
        hasStyle = true;
      }
    }

    // Extract image-specific props
    let data: Uint8Array | null = null;

    if (kind === "image") {
      const rawData = node.props.data;
      if (rawData instanceof Uint8Array) {
        data = rawData;
      }
      const objectFit = node.props.objectFit;
      if (typeof objectFit === "string") {
        params.object_fit = objectFit;
        hasParams = true;
      }
    }

    if (kind === "progressBar" && params.value !== undefined && params.progress === undefined) {
      params.progress = params.value;
      delete params.value;
    }

    return {
      kind,
      text,
      style: hasStyle ? style : null,
      params: hasParams ? params : null,
      data,
    };
  }

  function applyMountedProperty(node: HostElement, name: string, value: unknown): void {
    if (name === "children" || name === "ref" || name === "key" || name === "id") return;
    if (name === "type") return;
    if (isEventProp(name)) return;

    if (name === "style") {
      if (value && typeof value === "object") {
        runtime.ui.setStyle(node.widgetId, value as VellumStyle);
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

    if (name === "min" || name === "max" || name === "step" || name === "placeholder") {
      return;
    }

    if (isPrimitiveStyleValue(value)) {
      runtime.ui.setStyleProperty(node.widgetId, mapStyleKey(name), value);
    }

    if (name === "data" && value instanceof Uint8Array && runtime.ui.setImageData) {
      runtime.ui.setImageData(node.widgetId, value);
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
    runtime.ui.createWidget(node.widgetId, init.kind, parentWidgetId, init.text, init.style, init.params, init.data);
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
    const children: HostNode[] = [];
    let child = node.firstChild;
    while (child) {
      children.push(child);
      child = child.nextSibling;
    }

    for (let i = children.length - 1; i >= 0; i -= 1) {
      unmountSubtree(children[i]);
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
    const children: HostNode[] = [];
    let child = element.firstChild;
    while (child) {
      children.push(child);
      child = child.nextSibling;
    }

    for (let i = children.length - 1; i >= 0; i -= 1) {
      unmountSubtree(children[i]);
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
    const resolved = resolveChildValue(input);

    if (isHostNodeLike(resolved)) {
      return resolved;
    }

    if (typeof resolved === "string" || typeof resolved === "number") {
      return buildTextNode(String(resolved));
    }

    if (!isVellumJsxNode(resolved)) {
      return null;
    }

    const cached = jsxNodeMap.get(resolved as object);
    if (cached) {
      return cached;
    }

    const element = buildElementNode(resolved.type);
    jsxNodeMap.set(resolved as object, element);

    const props = resolved.props ?? {};
    for (const [key, value] of Object.entries(props)) {
      if (key === "children") continue;

      if (isReactiveAccessorProp(key, value)) {
        let prev: unknown = undefined;
        const setupEffect = () => {
          createRenderEffect(() => {
            const next = value();
            setElementProperty(element, key, next, prev);
            prev = next;
          });
        };

        if (resolved.owner) {
          runWithOwner(resolved.owner as Owner, setupEffect);
        } else {
          setupEffect();
        }
        continue;
      }

      setElementProperty(element, key, value, undefined);
    }

    const childrenProp = props.children;
    if (typeof childrenProp === "function" || hasDynamicChildren(childrenProp)) {
      const setupChildrenEffect = () => {
        createRenderEffect(() => {
          const resolvedChildren = typeof childrenProp === "function" ? childrenProp() : childrenProp;
          reconcileElementChildren(element, resolvedChildren);
        });
      };

      if (resolved.owner) {
        runWithOwner(
          resolved.owner as Owner,
          setupChildrenEffect
        );
      } else {
        setupChildrenEffect();
      }
    } else {
      reconcileElementChildren(element, childrenProp);
    }

    return element;
  }

  const renderer = createRenderer<HostNode | VellumRoot>({
    createElement(tag: string): HostElement {
      return buildElementNode(tag);
    },
    createTextNode(value: string): HostText {
      return buildTextNode(value);
    },
    replaceText(node: HostNode | VellumRoot, value: string): void {
      if (node.nodeType !== "text") return;

      node.text = String(value);
      if (node.mounted) {
        runtime.ui.setText(node.widgetId, node.text);
      }
    },
    setProperty(node: HostNode | VellumRoot, name: string, value: unknown, prev: unknown): void {
      if (node.nodeType !== "element") return;
      setElementProperty(node, name, value, prev);
    },
    insertNode(parent: HostNode | VellumRoot, node: HostNode | VellumRoot, anchor?: HostNode | VellumRoot): void {
      const hostParent = parent as HostParent;
      const hostNode = materializeHostNode(node);
      if (!hostNode) return;

      const hostAnchor = materializeHostNode(anchor) ?? null;
      insertHostNode(hostParent, hostNode, hostAnchor);
    },
    isTextNode(node: HostNode | VellumRoot): boolean {
      return node.nodeType === "text";
    },
    removeNode(parent: HostNode | VellumRoot, node: HostNode | VellumRoot): void {
      const hostParent = parent as HostParent;
      const hostNode = materializeHostNode(node);
      if (!hostNode) return;

      unlinkFromParent(hostParent, hostNode);
      unmountSubtree(hostNode);
    },
    getParentNode(node: HostNode | VellumRoot): HostParent | undefined {
      return node.parent ?? undefined;
    },
    getFirstChild(node: HostNode | VellumRoot): HostNode | undefined {
      return node.firstChild ?? undefined;
    },
    getNextSibling(node: HostNode | VellumRoot): HostNode | undefined {
      return node.nextSibling ?? undefined;
    },
  });

  function render(code: () => unknown, options?: RenderOptions | VellumRoot): VellumRoot {
    let root: VellumRoot;
    if (options && "nodeType" in options) {
      root = options;
    } else {
      const renderOptions = options as RenderOptions | undefined;
      root = createRoot(renderOptions?.parentId ?? DEFAULT_PARENT_ID);
    }

    renderer.render(code as () => HostNode | VellumRoot, root);
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
    createHostElement: (tag: string): VellumHostElement => buildElementNode(tag),
    createHostText: (value: string): VellumHostText => buildTextNode(value),
    setHostProperty: (node: VellumHostElement, name: string, value: unknown, prev?: unknown): void => {
      setElementProperty(node as HostElement, name, value, prev);
    },
    appendHostNode: (
      parent: VellumRoot | VellumHostElement,
      node: VellumHostElement | VellumHostText,
      anchor?: VellumHostElement | VellumHostText | null
    ): void => {
      insertHostNode(parent as HostParent, node as HostNode, (anchor as HostNode | null | undefined) ?? null);
    },
    render,
    dispose,
  };
}
