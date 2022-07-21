export declare enum OpenContext {
    Cli = 0,
    Dock = 1,
    Menu = 2,
    Dialog = 3,
    Desktop = 4
}
export interface WindowOpenable {
    folder_uri: string | null | undefined;
    file_uri: string | null | undefined;
}
export interface OpenConfiguration {
    label: string | null | undefined;
    url: string | null | undefined;
    uris_to_open: Array<WindowOpenable> | null | undefined;
    context_window_label: string | null | undefined;
    context: OpenContext;
    force_new_window: boolean;
    force_new_tabbed_window: boolean;
    force_reuse_window: boolean;
    force_empty_window: boolean;
    prefer_new_window: boolean;
    initial_startup: boolean;
    diff_mode: boolean;
}
export interface WindowSize {
    width: number;
    height: number;
}
export interface WindowPosition {
    x: number;
    y: number;
}
export declare enum Theme {
    Light = 0,
    Dark = 1
}
export interface EmptyWindowBackupInfo {
    backup_folder: string;
}
export interface FilesToOpen {
    files_to_open_or_create: Array<string>;
    files_to_diff: Array<string>;
    files_to_wait: Array<string>;
}
export interface TauriWindowOptions {
    label: string | null | undefined;
    url: string | null | undefined;
    always_on_top: boolean | null | undefined;
    center: boolean;
    decorations: boolean | null | undefined;
    focus: boolean;
    fullscreen: boolean | null | undefined;
    inner_size: WindowSize | null | undefined;
    max_inner_size: WindowSize | null | undefined;
    maximized: boolean | null | undefined;
    min_inner_size: WindowSize | null | undefined;
    position: WindowPosition | null | undefined;
    resizable: boolean | null | undefined;
    skip_taskbar: boolean | null | undefined;
    theme: Theme | null | undefined;
    title: string | null | undefined;
    transparent: boolean | null | undefined;
    visible: boolean | null | undefined;
}
export interface WindowOptions extends TauriWindowOptions {
    initial_startup: boolean;
    force_new_window: boolean;
    force_new_tabbed_window: boolean;
    force_reuse_window: boolean;
    force_empty_window: boolean;
    empty_window_backup_info: EmptyWindowBackupInfo | null | undefined;
    files_to_open: FilesToOpen;
    window_to_use: string | null | undefined;
    folder: string | null | undefined;
}
