import { OpenConfiguration, WindowOptions } from "./windows";
declare class Windows {
    constructor();
    open_window(configuration: OpenConfiguration): Promise<null>;
    open_empty_window(configuration: OpenConfiguration, options: WindowOptions): Promise<null>;
    open_existing_window(configuration: OpenConfiguration, windowToUse: string): Promise<null>;
    send_to_focused(channel: string, payload: any): Promise<null>;
    send_to_all(channel: string, payload: any): Promise<null>;
    get_focused_window(): Promise<string>;
    get_last_active_window(): Promise<string>;
}
declare const WindowsService: Windows;
export default WindowsService;
