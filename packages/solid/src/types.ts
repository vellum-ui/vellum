
import { Component } from "solid-js";
import type { JSX as SolidJSX } from "solid-js";

export type VellumStyle = Record<string, unknown>;

export interface VellumEvent {
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

export interface VellumRuntime {
  nextId?: () => string;
  ui: {
    createWidget: (
      id: string,
      kind: string,
      parentId: string | null,
      text: string | null,
      style: VellumStyle | null,
      params?: Record<string, unknown> | null,
      data?: Uint8Array | null
    ) => void;
    removeWidget: (id: string) => void;
    setText: (id: string, text: string) => void;
    setVisible: (id: string, visible: boolean) => void;
    setValue: (id: string, value: number) => void;
    setChecked: (id: string, checked: boolean) => void;
    setStyle: (id: string, style: VellumStyle) => void;
    setStyleProperty: (id: string, property: string, value: string | number | boolean) => void;
    setImageData?: (id: string, data: Uint8Array) => void;
    playVideo?: (id: string) => void;
    pauseVideo?: (id: string) => void;
    seekVideo?: (id: string, timeSecs: number) => void;
  };
  events: {
    on: (type: string, callback: (event: VellumEvent) => void) => () => void;
  };
}

export type WidgetActionHandler = (event: VellumEvent) => void;

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

export interface VellumJsxNode {
  __VellumJsx: true;
  type: string;
  props?: Record<string, unknown>;
  owner?: unknown;
}

export interface VellumRoot {
  nodeType: "root";
  parent: null;
  firstChild: HostNode | null;
  nextSibling: null;
  mounted: true;
  parentWidgetId: string | null;
}

export type HostParent = VellumRoot | HostElement;

export interface RenderOptions {
  parentId?: string | null;
}

export interface VellumHostElement {
  nodeType: "element";
  widgetId: string;
}

export interface VellumHostText {
  nodeType: "text";
  widgetId: string;
}

export interface VellumRenderer {
  createRoot(parentWidgetId?: string | null): VellumRoot;
  createHostElement(tag: string): VellumHostElement;
  createHostText(value: string): VellumHostText;
  setHostProperty(
    node: VellumHostElement,
    name: string,
    value: unknown,
    prev?: unknown
  ): void;
  appendHostNode(
    parent: VellumRoot | VellumHostElement,
    node: VellumHostElement | VellumHostText,
    anchor?: VellumHostElement | VellumHostText | null
  ): void;
  render(code: () => unknown, options?: RenderOptions | VellumRoot): VellumRoot;
  dispose(): void;
}

export interface VellumCommonProps {
  // Accessor form is supported for non-event props by renderer reactivity.
  // Keep this broad to avoid fighting editor JSX inference.
  id?: string;
  key?: string | number;
  ref?: (node: unknown) => void;
  style?: VellumStyle | (() => VellumStyle);
  text?: string | (() => string);
  value?: number | (() => number);
  checked?: boolean | (() => boolean);
  visible?: boolean | (() => boolean);
  onClick?: WidgetActionHandler;
  onValueChanged?: WidgetActionHandler;
  onHover?: WidgetActionHandler;
  onTextChanged?: WidgetActionHandler;
  onWidgetAction?: WidgetActionHandler;
  [key: string]: unknown;
}

export interface SliderProps extends VellumCommonProps {
  min?: number | (() => number);
  max?: number | (() => number);
  step?: number | (() => number);
  value?: number | (() => number);
}

export interface CheckboxProps extends VellumCommonProps {
  checked?: boolean | (() => boolean);
}

export interface ProgressBarProps extends VellumCommonProps {
  value?: number | (() => number);
}

export interface TextInputProps extends VellumCommonProps {
  placeholder?: string | (() => string);
}

export interface ImageProps extends VellumCommonProps {
  data?: Uint8Array | (() => Uint8Array);
  objectFit?: string | (() => string);
}

export interface VideoProps extends VellumCommonProps {
  src?: string | (() => string);
}

export interface HoverableProps extends VellumCommonProps {
  /** Hoverable accepts only a single child element. Wrap multiple children in a `<flex>` or `<row>`. */
  children?: SolidJSX.Element;
}

export interface FlexProps extends VellumCommonProps {
  // Assuming FlexProps is similar to VellumCommonProps for now,
  // as its definition was not provided in the original document or the instruction.
  // If it has specific properties, they should be added here.
}

export interface VellumIntrinsicElements {
  box: VellumCommonProps;
  flex: FlexProps;
  row: VellumCommonProps;
  column: VellumCommonProps;
  container: VellumCommonProps;
  sizedBox: VellumCommonProps;
  button: VellumCommonProps & { text?: string | (() => string); };
  iconButton: VellumCommonProps;
  label: VellumCommonProps;
  textInput: TextInputProps;
  textArea: VellumCommonProps;
  checkbox: CheckboxProps;
  progressBar: ProgressBarProps;
  spinner: VellumCommonProps;
  slider: SliderProps;
  svg: VellumCommonProps;
  image: ImageProps;
  prose: VellumCommonProps;
  grid: VellumCommonProps;
  stack: VellumCommonProps;
  hoverable: VellumCommonProps;
  video: VellumCommonProps & {
    src?: string | (() => string);
    playing?: boolean | (() => boolean);
    position?: number | (() => number);
  };
  [tag: string]: unknown;
}

// Types for JSX namespace
export namespace JSX {
  export interface IntrinsicElements extends VellumIntrinsicElements { }
}

