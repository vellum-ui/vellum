import { createComponent, getOwner } from "solid-js/dist/solid.js";
import type { Component, JSX as SolidJSX } from "solid-js";
import type { VellumCommonProps, VellumIntrinsicElements, VellumJsxNode } from "./types";

type JsxProps = Record<string, unknown> | null | undefined;

function normalizeChildren(children: unknown[]): unknown {
    if (children.length === 0) return undefined;
    if (children.length === 1) return children[0];
    return children;
}

function withChildren(props: JsxProps, children: unknown[]): Record<string, unknown> {
    const merged = props ? { ...props } : {};
    if (children.length > 0 && merged.children === undefined) {
        merged.children = normalizeChildren(children);
    }
    return merged;
}

function createJsxNode(type: string, props: Record<string, unknown>): VellumJsxNode {
    return {
        __VellumJsx: true,
        type,
        props,
        owner: getOwner(),
    };
}

export function Fragment(props: { children?: unknown }): unknown {
    return props.children ?? null;
}

export function jsx(type: unknown, props: JsxProps, ...children: unknown[]): SolidJSX.Element {
    const mergedProps = withChildren(props, children);

    if (typeof type === "function") {
        return createComponent(type as Component<Record<string, unknown>>, mergedProps);
    }

    return createJsxNode(String(type), mergedProps) as unknown as SolidJSX.Element;
}

export const jsxs = jsx;
export const jsxDEV = jsx;

export namespace JSX {
    export interface IntrinsicElements extends VellumIntrinsicElements { }
}
