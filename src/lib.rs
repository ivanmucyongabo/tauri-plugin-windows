//! Tauri Plugin for window management.
//!
//! This plugin provides interface for handling window state, basic file/folder handling, and window creation.
//! there are these advantages:
//! - Cache and backup window state
//! - Menu and event handlers for opening/closing windows files
//! - Robust window builder

use serde_json::Value as JsonValue;

use std::result::Result as StdResult;

use tauri::{
  plugin::{Plugin, Result as PluginResult},
  AppHandle, Invoke, Manager, PageLoadPayload, Result as TauriResult, RunEvent, Runtime, Window,
  WindowEvent, command
};

pub mod windows;
pub mod error;
pub mod event;

pub use windows::{
  OpenConfiguration,
  WindowOptions,
  WindowsSettings,
  WindowsAPI,
  WindowOpenable
};
pub use error::Error;

use windows::{
  LastActiveWindow,
  WindowsStateCache,
  WindowsBackupCache,
  WindowsRecentsCache,
  WindowStateTrait,
  WindowTrait
};
use event::{
  WINDOW_OPEN_FILES_EVENT,
  WINDOW_ADD_FOLDERS_EVENT,
  WINDOW_NEW_WINDOW_EVENT,
  WINDOW_OPEN_FILE_EVENT,
  WINDOW_OPEN_FOLDER_EVENT,
  WINDOW_CLOSE_WINDOW_EVENT,
  WINDOW_CLOSE_FILE_EVENT,
  WINDOW_CLOSE_FOLDER_EVENT,
};

type Result<T> = StdResult<T, String>;

#[command]
fn open_window<R: Runtime>(
  _app: AppHandle<R>,
  _window: Window<R>,
  configuration: OpenConfiguration,
) -> Result<()> {
  match WindowsAPI::open_window(&_app, configuration) {
    Ok(_created_window) => Ok(()),
    Err(e) => {
      eprintln!("Error: {:?}", e);
      Err(e.to_string())
    },
  }
}
#[command]
async fn open_empty_window<R: Runtime>(
  _app: AppHandle<R>,
  _window: Window<R>,
  configuration: OpenConfiguration,
  options: WindowOptions,
) -> Result<()> {
  match WindowsAPI::open_empty_window(&_app, configuration, options) {
    Ok(_created_window) => Ok(()),
    Err(e) => {
      eprintln!("Error: {:?}", e);
      Err(e.to_string())
    },
  }
}
#[command]
fn open_existing_window<R: Runtime>(
  _app: AppHandle<R>,
  _window: Window<R>,
  configuration: OpenConfiguration,
  _window_to_use: String,
) -> Result<()> {
  match WindowsAPI::open_existing_window(&_app, &_window, configuration) {
    Ok(_created_window) => Ok(()),
    Err(e) => {
      eprintln!("Error: {:?}", e);
      Err(e.to_string())
    },
  }
}
#[command]
fn send_to_focused(_channel: String) -> TauriResult<()> {
  Ok(())
}
#[command]
fn send_to_all(_channel: String, _window_labels_to_ignoree: Vec<String>) -> Result<()> {
  Ok(())
}
#[command]
fn get_focused_window() -> Result<()> {
  Ok(())
}
#[command]
fn get_last_active_window() -> Result<()> {
  Ok(())
}

pub struct TauriWindows<R: Runtime> {
  invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
  // plugin state, configuration fields
}

impl<R: Runtime> TauriWindows<R> {
  // you can add configuration fields here,
  // see https://doc.rust-lang.org/1.0.0/style/ownership/builders.html
  pub fn default() -> Self {
    Self {
      invoke_handler: Box::new(tauri::generate_handler![
        open_window,
        open_empty_window,
        open_existing_window,
        send_to_focused,
        send_to_all,
        get_focused_window,
        get_last_active_window
      ]),
    }
  }
}

impl<R: Runtime> Plugin<R> for TauriWindows<R> {
  /// The plugin name. Must be defined and used on the `invoke` calls.
  fn name(&self) -> &'static str {
    "windows"
  }

  /// The JS script to evaluate on initialization.
  /// Useful when your plugin is accessible through `window`
  /// or needs to perform a JS task on app initialization
  /// e.g. "window.awesomePlugin = { ... the plugin interface }"
  fn initialization_script(&self) -> Option<String> {
    None
  }

  /// initialize plugin with the config provided on `tauri.conf.json > plugins > $yourPluginName` or the default value.
  fn initialize(&mut self, app: &AppHandle<R>, _config: JsonValue) -> PluginResult<()> {
    app.manage(WindowsSettings::default());
    app.manage(WindowsStateCache::default());
    app.manage(WindowsBackupCache::default());
    app.manage(WindowsRecentsCache::default());

    Ok(())
  }

  /// Callback invoked when the Window is created.
  fn created(&mut self, _window: Window<R>) {
  }

  /// Callback invoked when the webview performs a navigation.
  fn on_page_load(&mut self, _window: Window<R>, _payloadd: PageLoadPayload) {}

  fn on_event(&mut self, app: &AppHandle<R>, event: &RunEvent) {
    match event {
      RunEvent::WindowEvent {
        label,
        event: WindowEvent::CloseRequested { api, .. },
        ..
      } => {
        let windows_state_cache = app.state::<WindowsStateCache>();

        match windows_state_cache.0.write() {
          Ok(mut cache) => {
            cache.handle_close_window(label)
          },
          Err(e) => {

          }
        };        
      }
      RunEvent::WindowEvent {
        label,
        event: WindowEvent::Resized(size),
        ..
      } => {
        if let Some(window) = app.get_window(label) {
          match window.emit("windows://resize", size) {
            Ok(_res) => {}
            Err(e) => {
              eprintln!("Error: {:?}", e)
            }
          }
        }
      },
      RunEvent::WindowEvent {
        label,
        event: WindowEvent::Destroyed,
        ..
      } => {
        let windows_state_cache = app.state::<WindowsStateCache>();

        match windows_state_cache.0.write() {
          Ok(mut cache) => {
            cache.handle_destroyed_window(label)
          },
          Err(e) => {

          }
        };
      },
      RunEvent::WindowEvent {
        label,
        event: WindowEvent::Focused(focus),
        ..
      } => {
        let windows_state_cache = app.state::<WindowsStateCache>();

        match windows_state_cache.0.write() {
          Ok(mut cache) => {
            cache.handle_focused_window(label, focus)
          },
          Err(e) => {

          }
        };
      },
      RunEvent::ExitRequested { api, .. } => {
        // Prevents the app from exiting.
        // This will cause the core thread to continue running in the background even without any open windows.
        // api.prevent_exit();
      },
      // Ignore all other cases.
      _ => {}
    }
  }

  /// Extend the invoke handler.
  fn extend_api(&mut self, invoke: Invoke<R>) {
    (self.invoke_handler)(invoke)
  }
}
