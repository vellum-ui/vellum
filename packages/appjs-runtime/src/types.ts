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
