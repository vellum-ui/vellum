import type { BoxStyle } from "../types.ts";

export type LabelStyle = BoxStyle;
export type ButtonStyle = BoxStyle;
export type SvgStyle = BoxStyle;
export type ImageStyle = BoxStyle;
export type FlexStyle = BoxStyle;
export type SizedBoxStyle = BoxStyle;
export type CheckboxStyle = BoxStyle;
export type TextInputStyle = BoxStyle;
export type TextAreaStyle = BoxStyle;
export type ProseStyle = BoxStyle;
export type ProgressBarStyle = BoxStyle;
export type SpinnerStyle = BoxStyle;
export type SliderStyle = BoxStyle;
export type ZStackStyle = BoxStyle;
export type PortalStyle = BoxStyle;

export interface ButtonData {
    svgData?: string;
}

export interface SvgData {
    svgData?: string;
}

export interface ImageData {
    object_fit?: string;
}

export interface CheckboxData {
    checked: boolean;
}

export interface TextInputData {
    placeholder?: string;
}

export interface ProgressBarData {
    progress?: number;
}

export interface SliderData {
    minValue: number;
    maxValue: number;
    value: number;
    step?: number;
}
