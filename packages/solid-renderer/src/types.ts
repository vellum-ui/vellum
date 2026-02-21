
import { Component } from "solid-js";

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
      style: AppJsStyle | null,
      params?: Record<string, unknown> | null,
      data?: Uint8Array | null
    ) => void;
    removeWidget: (id: string) => void;
    setText: (id: string, text: string) => void;
    setVisible: (id: string, visible: boolean) => void;
    setValue: (id: string, value: number) => void;
    setChecked: (id: string, checked: boolean) => void;
    setStyle: (id: string, style: AppJsStyle) => void;
    setStyleProperty: (id: string, property: string, value: string | number | boolean) => void;
    setImageData?: (id: string, data: Uint8Array) => void;
  };
  events: {
    on: (type: string, callback: (event: AppJsEvent) => void) => () => void;
  };
}

export type WidgetActionHandler = (event: AppJsEvent) => void;

export type HostNode = HostElement | HostText;

export interface HostCommon {
  parent: HostParent | null;
  firstChild: HostNode | null;
  nextSibling: HostNode | null;
  mounted: boolean;
}

export interface HostElement extends HostCommon {
  nodeType: "element";
  tag: string;
  widgetId: string;
  props: Record<string, unknown>;
  handlers: Map<string, Set<WidgetActionHandler>>;
}

export interface HostText extends HostCommon {
  nodeType: "text";
  widgetId: string;
  text: string;
}

export interface AppJsJsxNode {
  __appjsJsx: true;
  type: string;
  props?: Record<string, unknown>;
  owner?: unknown;
}

export interface AppJsRoot {
  nodeType: "root";
  parent: null;
  firstChild: HostNode | null;
  nextSibling: null;
  mounted: true;
  parentWidgetId: string | null;
}

export type HostParent = AppJsRoot | HostElement;

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

export interface SliderProps extends AppJsCommonProps {
  min?: number | (() => number);
  max?: number | (() => number);
  step?: number | (() => number);
  value?: number | (() => number);
}

export interface CheckboxProps extends AppJsCommonProps {
  checked?: boolean | (() => boolean);
}

export interface ProgressBarProps extends AppJsCommonProps {
  value?: number | (() => number);
}

export interface TextInputProps extends AppJsCommonProps {
  placeholder?: string | (() => string);
}

export interface ImageProps extends AppJsCommonProps {
  data?: Uint8Array | (() => Uint8Array);
  objectFit?: string | (() => string);
}

export type AppJsIntrinsicElements = {
  [tagName: string]: AppJsCommonProps;
  label: AppJsCommonProps;
  button: AppJsCommonProps;
  checkbox: CheckboxProps;
  textInput: TextInputProps;
  slider: SliderProps;
  progressBar: ProgressBarProps;
  spinner: AppJsCommonProps;
  prose: AppJsCommonProps;
  flex: AppJsCommonProps;
  row: AppJsCommonProps;
  column: AppJsCommonProps;
  box: AppJsCommonProps;
  zstack: AppJsCommonProps;
  portal: AppJsCommonProps;
  image: ImageProps;
};

// Types for JSX namespace
export namespace JSX {
  export interface IntrinsicElements extends AppJsIntrinsicElements { }
}

