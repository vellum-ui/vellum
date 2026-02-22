export type FlexDirection = "row" | "column";
export type CrossAlign = "start" | "center" | "end" | "fill" | "baseline";
export type MainAlign =
    | "start"
    | "center"
    | "end"
    | "space-between"
    | "space-around"
    | "space-evenly";

export interface BoxStyle {
    fontSize?: number;
    fontWeight?: number | string;
    fontStyle?: "normal" | "italic";
    fontFamily?: string;
    color?: string;
    letterSpacing?: number;
    lineHeight?: number;
    wordSpacing?: number;
    underline?: boolean;
    strikethrough?: boolean;
    textAlign?: "start" | "center" | "end" | "justify" | "left" | "right";

    background?: string;
    backgroundColor?: string;
    borderColor?: string;
    hoveredBorderColor?: string;
    hoverBorderColor?: string;
    borderWidth?: number;
    cornerRadius?: number;
    borderRadius?: number;
    padding?: number | string;
    width?: number;
    height?: number;

    flex?: number;
    direction?: FlexDirection;
    crossAxisAlignment?: CrossAlign;
    mainAxisAlignment?: MainAlign;
    gap?: number;
    mustFillMainAxis?: boolean;

    [key: string]: unknown;
}

export type VellumStyle = BoxStyle;

export interface ButtonParams {
    svgData?: string;
}

export interface SvgParams {
    svgData?: string;
}

export interface ImageParams {
    object_fit?: string;
}

export interface CheckboxParams {
    checked: boolean;
}

export interface TextInputParams {
    placeholder?: string;
}

export interface ProgressBarParams {
    progress?: number;
}

export interface SliderParams {
    minValue: number;
    maxValue: number;
    value: number;
    step?: number;
}

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
