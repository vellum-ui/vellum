
import { VellumStyle, VellumJsxNode, HostNode, HostParent } from "./types";

export function isEventProp(name: string): boolean {
  return /^on[A-Z]/.test(name);
}

export function normalizeEventName(propName: string): string | null {
  if (!isEventProp(propName)) return null;
  const raw = propName.slice(2);
  if (!raw) return null;
  const normalized = raw.charAt(0).toLowerCase() + raw.slice(1);
  return normalized;
}

export function normalizeWidgetKind(tag: string): string {
  switch (tag) {
    case "iconButton":
    case "icon_button":
      return "button";
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

export function mapStyleKey(name: string): string {
  if (name === "min") return "minValue";
  if (name === "max") return "maxValue";
  if (name === "className") return "class";
  return name;
}

export function isNullish(value: unknown): value is null | undefined {
  return value === null || value === undefined;
}

export function isPrimitiveStyleValue(value: unknown): value is string | number | boolean {
  return (
    typeof value === "string" ||
    typeof value === "number" ||
    typeof value === "boolean"
  );
}

export function unlinkFromParent(parent: HostParent, node: HostNode): void {
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

export function linkIntoParent(parent: HostParent, node: HostNode, anchor: HostNode | null): void {
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

export function createEmptyStyle(): VellumStyle {
  return Object.create(null) as VellumStyle;
}

export function isVellumJsxNode(value: unknown): value is VellumJsxNode {
  return (
    typeof value === "object" &&
    value !== null &&
    "__VellumJsx" in value &&
    (value as { __VellumJsx?: unknown }).__VellumJsx === true &&
    typeof (value as { type?: unknown }).type === "string"
  );
}

export function isHostNodeLike(value: unknown): value is HostNode {
  return (
    typeof value === "object" &&
    value !== null &&
    "nodeType" in value &&
    (((value as { nodeType?: unknown }).nodeType === "element") ||
      ((value as { nodeType?: unknown }).nodeType === "text"))
  );
}

export function resolveChildValue(input: unknown, maxDepth = 32): unknown {
  let value = input;
  let depth = 0;

  while (typeof value === "function" && depth < maxDepth) {
    value = (value as () => unknown)();
    depth += 1;
  }

  return value;
}

export function normalizeChildrenArray(input: unknown): unknown[] {
  const resolved = resolveChildValue(input);

  if (Array.isArray(resolved)) {
    const out: unknown[] = [];
    for (const entry of resolved) {
      out.push(...normalizeChildrenArray(entry));
    }
    return out;
  }

  if (resolved === null || resolved === undefined || resolved === false || resolved === true) {
    return [];
  }

  return [resolved];
}

export function isReactiveAccessorProp(name: string, value: unknown): value is () => unknown {
  if (typeof value !== "function") return false;
  if (name === "children" || name === "ref" || name === "key") return false;
  if (isEventProp(name)) return false;
  return true;
}
