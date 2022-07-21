import { invoke } from "@tauri-apps/api/tauri";
import { OpenConfiguration, WindowOptions } from "./windows";

class Windows {
  constructor() {}

  open_window(configuration: OpenConfiguration): Promise<null> {
    return invoke("plugin:windows|open_window", {
      configuration,
    });
  }

  open_empty_window(
    configuration: OpenConfiguration,
    options: WindowOptions
  ): Promise<null> {
    return invoke("plugin:windows|open_empty_window", {
      configuration,
      options,
    });
  }

  open_existing_window(
    configuration: OpenConfiguration,
    windowToUse: string
  ): Promise<null> {
    return invoke("plugin:windows|open_existing_window", {
      configuration,
      windowToUse,
    });
  }

  send_to_focused(channel: string, payload: any): Promise<null> {
    return invoke("plugin:windows|send_to_focused", {
      channel,
      payload,
    });
  }

  send_to_all(channel: string, payload: any): Promise<null> {
    return invoke("plugin:windows|send_to_all", {
      channel,
      payload,
    });
  }

  get_focused_window(): Promise<string> {
    return invoke("plugin:windows|get_focused_window");
  }

  get_last_active_window(): Promise<string> {
    return invoke("plugin:windows|get_last_active_window");
  }
}

const WindowsService = new Windows();
Object.freeze(WindowsService);

export default WindowsService;
