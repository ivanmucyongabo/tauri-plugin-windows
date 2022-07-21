//! Windows.
//!
//! This module contains methods and types for window creation. 

use std::{
  convert::Into,
  path::PathBuf,
  result::Result as StdResult,
  sync::{
    atomic::{AtomicUsize, Ordering},
    RwLock
  },
  time::Duration,
};

use serde::{Deserialize, Serialize};
use serde_json::to_string;

use tauri::{
  window::WindowBuilder, Error as TauriError, Manager, Runtime, State, Theme, Window, WindowUrl,
};

mod menu;
pub use menu::Menu;

mod window;
pub use window::{
  EmptyWindowBackupInfo,
  FolderBackupInfo,
  LastActiveWindow,
  RecentPath,
  RecentFile,
  RecentFolder,
  WindowConfiguration,
  WindowsStateCache,
  WindowsBackupCache,
  WindowsRecentsCache,
  WindowState,
  WindowStateTrait,
  WindowTrait
};

use crate::error::Error;

use crate::event::{
  WINDOW_OPEN_FILES_EVENT,
  WINDOW_ADD_FOLDERS_EVENT
};

static COUNTER: AtomicUsize = AtomicUsize::new(1);

/// Open window request source.
#[derive(Deserialize, PartialEq)]
pub enum OpenContext {
  /// Opening through the API.
  Api,

  /// Opening when running from the command line.
  Cli,

  /// macOS only: Opening from the dock (also when opening files to a running instance from desktop).
  Dock,

  /// Opening from the main application window.
  Menu,

  /// Opening from a file or folder dialog.
  Dialog,

  /// Opening from the OS's UI.
  Desktop,
}

impl Default for OpenContext {
  fn default() -> Self {
    OpenContext::Desktop
  }
}

/// Data type for openable resources.
/// 
/// Holds PathBufs for folder or file uri.
#[derive(Default, Deserialize)]
pub struct WindowOpenable {
  pub folder: Option<PathBuf>,
  pub file: Option<PathBuf>,
}

impl WindowOpenable {
  pub fn new(file: Option<PathBuf>, folder: Option<PathBuf>) -> Self {
    Self { folder, file }
  } 
}

/// Date type for files to be opened.
/// 
/// Holds vectors of PathBufs for files to open, diff, or wait.
#[derive(Clone, Default, Deserialize)]
pub struct FilesToOpen {
  pub files_to_open_or_create: Vec<PathBuf>
}

/// Data type for file type
#[derive(Clone, Deserialize, PartialEq, Serialize)]
pub enum FileType {
  Directory,
  File,
}

impl Default for FileType {
  fn default() -> Self {
    FileType::File
  }
}

/// Data type for info about path to open.
/// 
/// Contains info about resources available, backup, and state.
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct PathToOpen {
  pub folder: Option<PathBuf>,
  pub file: Option<PathBuf>,
  pub backup_path: Option<PathBuf>,
  pub path_type: FileType,
  pub exists: bool,
  pub window: Option<String>,
  pub label: Option<String>,
}

/// Data type for open options.
/// 
/// Contains data about whether a new window should be used to open resources.
#[derive(Default)]
struct OpenOptions {
  pub open_folder_in_new_window: bool,
  pub open_files_in_new_window: bool,
}

#[derive(Default, Deserialize)]
pub struct WindowSize {
  pub width: f64,
  pub height: f64,
}

#[derive(Default, Deserialize)]
pub struct WindowPosition {
  pub x: f64,
  pub y: f64,
}

/// Configuration for window creation used by api.
#[derive(Default, Deserialize)]
pub struct OpenConfiguration {
  pub label: Option<String>,
  pub url: Option<WindowUrl>,
  pub uris_to_open: Option<Vec<WindowOpenable>>,
  pub context_window_label: Option<String>,
  pub context: OpenContext,
  pub force_new_window: bool,
  pub force_new_tabbed_window: bool,
  pub force_reuse_window: bool,
  pub force_empty_window: bool,
  pub prefer_new_window: bool,
  pub initial_startup: bool,
  pub diff_mode: bool,
}

/// Options for window creation used by api.
#[derive(Default, Deserialize)]
pub struct WindowOptions {
  pub label: Option<String>,
  pub url: Option<WindowUrl>,
  pub always_on_top: Option<bool>,
  pub center: bool,
  pub decorations: Option<bool>,
  pub focus: bool,
  pub fullscreen: Option<bool>,
  pub inner_size: Option<WindowSize>,
  pub max_inner_size: Option<WindowSize>,
  pub maximized: Option<bool>,
  pub min_inner_size: Option<WindowSize>,
  pub position: Option<WindowPosition>,
  pub resizable: Option<bool>,
  pub skip_taskbar: Option<bool>,
  pub theme: Option<Theme>,
  pub title: Option<String>,
  pub transparent: Option<bool>,
  pub visible: Option<bool>,
  pub initial_startup: bool,
  pub force_new_window: bool,
  pub force_new_tabbed_window: bool,
  pub force_reuse_window: bool,
  pub force_empty_window: bool,
  pub empty_window_backup_info: Option<EmptyWindowBackupInfo>,
  pub files_to_open: FilesToOpen,
  pub window_to_use: Option<String>,
  pub folder: Option<PathBuf>,
}

// Payloads
/// Payload for add folder global event.
#[derive(Clone, Serialize)]
pub struct AddFolderPayload {
  pub folders_to_add: Vec<PathToOpen>,
}

/// Payload for open files global event.
#[derive(Clone, Serialize)]
pub struct OpenFilePayload {
  pub files_to_open_or_create: Vec<PathBuf>,
}

// Managed States
#[derive(Clone, PartialEq)]
pub enum OpenInNewWindow {
  On,
  Off,
  Default,
}

impl Default for OpenInNewWindow {
  fn default() -> Self {
    OpenInNewWindow::Default
  }
}

#[derive(PartialEq)]
pub enum RestoreWindows {
  Preserve,
  All,
  Folders,
  One,
  None,
}

impl Default for RestoreWindows {
  fn default() -> Self {
    RestoreWindows::All
  }
}

/// Dimension type for new windows.
/// 
/// Flags for new window dimensions.
#[derive(PartialEq)]
pub enum NewWindowDimensions {
  /// Use system default dimensions for window size.
  Default,
  /// Use calling window dimensions for window size.
  Inherit,
  /// Use a relative offset for window size.
  Offset,
  /// Use maximum dimensions for window size.
  Maximized,
  /// Use fullscreen for window size.
  Fullscreen,
}

impl Default for NewWindowDimensions {
  fn default() -> Self {
    NewWindowDimensions::Default
  }
}

#[derive(Default)]
pub struct InnerWindowsSettings {
  pub open_files_in_new_window: OpenInNewWindow,
  pub open_folders_in_new_window: OpenInNewWindow,
  pub open_without_arguments_in_new_window: OpenInNewWindow,
  pub restore_windows: RestoreWindows,
  pub restore_fullscreen: bool,
  pub zoom_level: f64,
  pub new_window_dimensions: NewWindowDimensions,
  pub native_tabs: bool,
  pub native_full_screen: bool,
  pub close_when_empty: bool,
  pub click_through_inactive: bool,
}

/// Managed state for window settings.
/// 
/// Managed state that can be set through tauri.config.json
/// and updated during runtime. 
#[derive(Default)]
pub struct WindowsSettings(pub RwLock<InnerWindowsSettings>);

pub type Result<T> = StdResult<T, Error>;

// Endpoints for creating resourse data structures

/// Create [`PathToOpen`] from [`PathBuf`].
fn resolve_file_path(path: &PathBuf) -> Option<PathToOpen> {
  if !path.exists() {
    return Some(PathToOpen {
      file: Some(path.clone()),
      path_type: FileType::File,
      exists: false,
      ..Default::default()
    });
  } else if path.is_file() {
    return Some(PathToOpen {
      file: Some(path.clone()),
      path_type: FileType::File,
      exists: true,
      ..Default::default()
    });
  } else if path.is_dir() {
    return Some(PathToOpen {
      folder: Some(path.clone()),
      path_type: FileType::Directory,
      exists: true,
      ..Default::default()
    });
  }

  return Some(PathToOpen {
    file: Some(path.clone()),
    path_type: FileType::File,
    exists: true,
    ..Default::default()
  });
}

/// Create [`PathToOpen`] from [`WindowOpenable`].
fn resolve_openable(openable: &WindowOpenable) -> Option<PathToOpen> {
  let uri: &PathBuf;

  if openable.folder.is_some() {
    uri = openable.folder.as_ref().unwrap();
  } else {
    uri = openable.file.as_ref().unwrap();
  }

  resolve_file_path(uri)
}

/// Extracts PathToOpen from uris.
fn extract_paths(uris_to_open: &Option<Vec<WindowOpenable>>) -> Vec<PathToOpen> {
  let mut paths_to_open: Vec<PathToOpen> = Vec::new();

  if let Some(uris) = uris_to_open {
    for path_to_open in uris {
      if let Some(path) =resolve_openable(path_to_open) {
        paths_to_open.push(path);
      }
    }
  }

  paths_to_open
}

// Endpoint for creating window ids/labels

/// Create a random number.
fn get_id() -> usize {
  return COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// Create a randon window label.
/// 
/// Attaches windows_ to a random number, making sure labels are unique.
fn new_window_label() -> String {
  let id = get_id().to_string();
  let mut label = String::from("windows_");

  label.push_str(&id);

  return label;
}

/// Get new window creation options.
/// 
/// Uses window configuration and window settings
/// to derive whether a new window should be opened.
fn should_open_new_window<'a, R: Runtime, M: Manager<R>>(
  manager: &'a M,
  configuration: &OpenConfiguration,
) -> OpenOptions {
  // let the user settings override how folders are open in a new window or same window unless we are forced
  let window_settings = manager.state::<WindowsSettings>();

  let open_options = match window_settings.0.read() {
    Ok(settings) => {
      let mut open_folder_in_new_window = (configuration.prefer_new_window || configuration.force_new_window) && !configuration.force_reuse_window;
      let mut open_files_in_new_window = false;

      if !configuration.force_new_window
        && !configuration.force_reuse_window
        && (settings.open_folders_in_new_window == OpenInNewWindow::On || settings.open_folders_in_new_window == OpenInNewWindow::Off)
      {
        open_folder_in_new_window = settings.open_folders_in_new_window == OpenInNewWindow::On
      }

      if configuration.force_new_window || configuration.force_reuse_window {
        open_files_in_new_window = !!configuration.force_new_window && !configuration.force_reuse_window;
      }
      else {
        // Linux/Windows: by default we open files in the new window unless triggered via DIALOG / MENU context
        // or from the integrated terminal where we assume the user prefers to open in the current window
        if configuration.context != OpenContext::Dialog && configuration.context != OpenContext::Menu {
          open_files_in_new_window = true;
        }
        // finally check for overrides of default
        if settings.open_files_in_new_window == OpenInNewWindow::On || settings.open_files_in_new_window == OpenInNewWindow::Off {
          open_files_in_new_window = settings.open_files_in_new_window == OpenInNewWindow::On;
        }
      }

      OpenOptions {
        open_folder_in_new_window,
        open_files_in_new_window
      } 
    },
    Err(e) => {
      OpenOptions {
        open_folder_in_new_window: (configuration.prefer_new_window || configuration.force_new_window) && !configuration.force_reuse_window,
        open_files_in_new_window: false
      }    
    }
  };

  open_options
}

// Endpoints for get window by resource type

/// Check if folder uri is open in an existing window.
/// 
/// Uses [`WindowsStateCache`] to find if a window containing the current folder uri.
fn find_window_on_folder<'a, R: Runtime, M: Manager<R>>(
  manager: &'a M,
  folder: Option<&PathBuf>,
) -> Option<String> {
  if let Some(cmp_uri) = folder {
    let windows_state_cache = manager.state::<WindowsStateCache>();
  
    let res = match windows_state_cache.0.read() {
      Ok(cache) => {
        // Then go with single folder windows that are parent of the provided file path
        match cache.state().opened_windows
          .iter()
          .find(|(_label, window_state)| {
            if let Some(uri) = &window_state.folder {
              cmp_uri.eq(uri)
            }
            else {
              false
            }
          }) {
            Some((label, _window_state)) => Some(label.to_string()),
            None => None
          }        
      },
      Err(e) => {
        None
      }
    };

    res
  }
  else {
    None
  }
}

/// Check if file uri is open in an existing window.
///
/// Uses [`WindowsStateCache`] to find if a window containing the current file uri.
fn find_window_on_file<'a, R: Runtime, M: Manager<R>>(
  manager: &'a M,
  file: &PathBuf,
) -> Option<Window<R>> {
  let windows_state_cache = manager.state::<WindowsStateCache>();

  let res = match windows_state_cache.0.read() {
    Ok(cache) => {
      // Then go with single folder windows that are parent of the provided file path
      let single_folder_windows_on_file_path: Vec<(&String, &WindowState)> = cache.state().opened_windows
      .iter()
      .filter(|(_label, window_state)| {
        if let Some(parent) = window_state.folder.as_ref() {
          // check if equal or parent
          file.eq(parent) || file.starts_with(parent)
        }
        else {
          false
        }
      })
      .collect();

      if single_folder_windows_on_file_path.len() > 0 {
        manager.get_window(&single_folder_windows_on_file_path[0].0)
      }
      else {
        None
      }
    },
    Err(e) => {
      None
    }
  };

  res
}

// Endpoints for getting managed state

/// Get window settings.
///
/// Gets window settings from managed state.
fn get_window_config<'a, R: Runtime, M: Manager<R>>(manager: &'a M) -> State<'_, WindowsSettings> {
  return manager.state::<WindowsSettings>();
}

// Endpoints for resource paths


fn get_paths_from_last_session<'a, R: Runtime, M: Manager<R>>(manager: &'a M) -> Vec<PathToOpen> {
  let window_settings = manager.state::<WindowsSettings>();

  let paths = match window_settings.0.read() {
    Ok(settings) => {
      match settings.restore_windows {
        // none: no window to restore
        RestoreWindows::None => Vec::new(),
        // one: restore last opened workspace/folder or empty window
        // all: restore all windows
        // folders: restore last opened folders only
        RestoreWindows::One
        | RestoreWindows::All
        | RestoreWindows::Preserve
        | RestoreWindows::Folders => {
          // Collect previously opened windows
          let mut last_session_windows = Vec::new();
          let windows_state_cache = manager.state::<WindowsStateCache>();

          let paths_from_last_session = match windows_state_cache.0.read() {
            Ok(cache) => {
              if settings.restore_windows != RestoreWindows::One {
                last_session_windows.append(
                  &mut cache.state().opened_windows
                    .iter()
                    .filter_map(|(label, window_state)| manager.get_window(&*label))
                    .collect()
                );
              }

              let last_active_window = cache.state().last_active_window.as_ref().map_or(
                None, 
                |v| manager.get_window(&*v.label)
              );
              
              if let Some(window) = last_active_window {
                last_session_windows.push(window);
              }
    
              let mut paths_to_open: Vec<PathToOpen> = Vec::new();              

              for last_session_window in &last_session_windows {
                let window_state = cache.get_item(last_session_window.label());

                if let Some(state) = window_state {
                  // Folders
                  if state.folder.is_some() {
                    let path_to_open: Option<PathToOpen> = resolve_openable(&WindowOpenable {
                      folder: state.folder.clone(),
                      ..Default::default()
                    });
    
                    if path_to_open.is_some() {
                      paths_to_open.push(path_to_open.unwrap());
                    }
                  }
                  // Empty window, potentially editors open to be restored
                  else if settings.restore_windows != RestoreWindows::Folders {
                    paths_to_open.push(PathToOpen {
                      backup_path: state.backup_path.clone(),
                      ..Default::default()
                    });
                  }
                }
              }
            
              paths_to_open
            },
            Err(e) => {
              Vec::new()
            }
          };

          paths_from_last_session
        }
      }
    },
    Err(e) => {
      Vec::new()
    }
  };

  paths
}

fn get_empty_window_backup_paths<'a, R: Runtime, M: Manager<R>>(
  manager: &'a M,
) -> Vec<EmptyWindowBackupInfo> {
  let windows_backup_cache = manager.state::<WindowsBackupCache>();

  let empty_windows = match windows_backup_cache.0.read() {
    Ok(cache) => {
      cache.backups.empty_windows.to_vec()
    },
    Err(e) => {
      Vec::new()
    }
  };

  empty_windows
}

fn get_paths_to_open<'a, R: Runtime, M: Manager<R>>(
  manager: &'a M,
  uris_to_open: &Option<Vec<WindowOpenable>>,
  force_empty_window: &bool,
  initial_startup: &bool,
) -> Vec<PathToOpen> {
  let mut paths_to_open: Vec<PathToOpen> = Vec::new();
  let mut restored_windows = false;

  // Extract paths: from API
  if uris_to_open.is_some() {
    if uris_to_open.as_ref().unwrap().len() > 0 {
      paths_to_open = extract_paths(uris_to_open);
    }
  }
  // Check for force empty
  else if *force_empty_window {
    paths_to_open = Vec::new();
  }
  // Extract paths: from previous session
  else {
    paths_to_open = get_paths_from_last_session(manager);

    if paths_to_open.len() == 0 {
      paths_to_open.push(PathToOpen {
        ..Default::default()
      }); // add an empty window if we did not have windows to restore
    }
    restored_windows = true;
  }

  // Check for `window.startup` setting to include all windows
  // from the previous session if this is the initial startup and we have
  // not restored windows already otherwise.
  // Use `unshift` to ensure any new window to open comes last
  // for proper focus treatment.
  if *initial_startup && !restored_windows {
    let mut windows_from_previos_session = get_paths_from_last_session(manager);

    windows_from_previos_session.retain(|path| path.backup_path.is_some());

    paths_to_open.splice(1..1, windows_from_previos_session);
  }

  return paths_to_open;
}

// Endpoints for retriving stateful windows
fn get_focused_window<'a, R: Runtime, M: Manager<R>>(manager: &'a M) -> Option<Window<R>> {
  None
}

fn get_last_active_window<'a, R: Runtime, M: Manager<R>>(manager: &'a M) -> Option<Window<R>> {
  let windows_state_cache = manager.state::<WindowsStateCache>();

  let res = match windows_state_cache.0.read() {
    Ok(cache) => {
      let last_focused_date = cache.state().opened_windows
      .iter()
      .map(|(_label, window)| window.last_focus_time)
      .max()
      .unwrap_or(Duration::new(0, 0));
  
      let state = cache.state().opened_windows
        .iter()
        .find(|(_label, window_state)| window_state.last_focus_time == last_focused_date);
    
      if let Some(s) = state {
        manager.get_window(&s.0)
      }
      else {
        None
      }
    },
    Err(e) => {
      None
    }
  };

  res
}

// Endpoints for adding resources to windows
fn open_files_in_existing_window<'a, R: Runtime>(
  _configuration: &OpenConfiguration,
  window: &Window<R>,
  files_to_open: &FilesToOpen,
) -> Result<()> {
  window.set_focus().map_err(|e| Error::Tauri(e))?; // make sure window has focus

  match to_string(&OpenFilePayload {
    files_to_open_or_create: files_to_open.files_to_open_or_create.clone(),
  }) {
    Ok(serialized_payload) => {
      window.trigger_global(WINDOW_OPEN_FILES_EVENT, Some(serialized_payload));
      Ok(())
    },
    Err(e) => Err(Error::SerdeJson(e))
  }
}

fn add_folders_to_existing_window<'a, R: Runtime>(
  window: &Window<R>,
  folders_to_add: &Vec<PathToOpen>,
) -> Result<()> {
  window.set_focus().map_err(|e| Error::Tauri(e))?; // make sure window has focus

  match to_string(&AddFolderPayload {
    folders_to_add: folders_to_add.clone(),
  }) {
    Ok(serialized_payload) => {
      window.trigger_global(WINDOW_ADD_FOLDERS_EVENT, Some(serialized_payload));
      Ok(())
    },
    Err(e) => Err(Error::SerdeJson(e))
  }

}

// Endpoints for creating different window states
fn open_in_webview_window<'a, R: Runtime, M: Manager<R>>(
  manager: &'a M,
  options: WindowOptions,
) -> Result<Window<R>> {
  let _window_config = manager.state::<WindowsSettings>();
  let windows_backup_cache = manager.state::<WindowsBackupCache>();

  // Build up the window configuration from provided options, config and environment
  let mut configuration: WindowConfiguration = WindowConfiguration {
    backup_path: None,
    cache_path: None,
    files_to_open_or_create: options.files_to_open.files_to_open_or_create,
    is_initial_startup: options.initial_startup,
    full_screen: options.fullscreen.unwrap_or(false),
    maximized: options.maximized.unwrap_or(false),
    folder: options.folder,
    // home_dir: ,
    // tmp_dir: ,
    // user_data_dir: ,   
    ..Default::default() 
  };

  let mut window: Option<Window<R>> = None;

  if !options.force_new_window && !options.force_new_tabbed_window {
    window = match options.window_to_use.is_some() {
      true => manager.get_window(&options.window_to_use.unwrap()),
      false => get_last_active_window(manager),
    };

    if let Some(window_to_focus) = &window {
      window_to_focus.set_focus()?;
    }
  }

  // Existing window
  if let Some(existing_window) = window {
    match windows_backup_cache.0.write() {
      Ok(mut cache) => {
        if let Some(folder) = &configuration.folder {
          configuration.backup_path = Some(cache.add_folder_backup(folder, existing_window.label()));
        } else {
          let backup_folder  = options.empty_window_backup_info
          .and_then(|info| info.backup_folder);
    
          configuration.backup_path = Some(cache.add_empty_window_backup(&backup_folder, existing_window.label()));
        }

        Ok(existing_window)
      },
      Err(e) => {
        Err(Error::RwLock(e.to_string()))
      }
    }
  }
  // New window
  else {
    let unique_label = options.label.unwrap_or(new_window_label());
    let url = options.url.unwrap_or(WindowUrl::App("index.html".into()));
    let handle = manager.app_handle();
    // Create the window
    let window_builder = WindowBuilder::new(&handle, unique_label, url)
      .always_on_top(options.always_on_top.unwrap_or(false)) // Whether the window should always be on top of other windows.
      .decorations(options.decorations.unwrap_or(true)) // Whether the window should have borders and bars.
      .fullscreen(options.fullscreen.unwrap_or(false)) // Whether to start the window in fullscreen or not.
      .maximized(options.maximized.unwrap_or(false)) // Whether the window should be maximized upon creation.
      .resizable(options.resizable.unwrap_or(true)) // Whether the window is resizable or not.
      .skip_taskbar(options.skip_taskbar.unwrap_or(false)) // Sets whether or not the window icon should be added to the taskbar.
      .theme(options.theme) // Forces a theme or uses the system settings if None was provided.
      .title(options.title.unwrap_or(String::from(""))) // The title of the window in the title bar.
      .transparent(options.transparent.unwrap_or(false)) // Whether the the window should be transparent.
      .visible(options.visible.unwrap_or(true)); // Whether the window should be immediately visible upon creation.

    match window_builder.build() {
      Ok(created_window) => {
        match windows_backup_cache.0.write() {
          Ok(mut cache) => {
            if let Some(folder) = &configuration.folder {
              configuration.backup_path = Some(cache.add_folder_backup(folder, created_window.label()));
            } else {
              let backup_folder  = options.empty_window_backup_info
              .and_then(|info| info.backup_folder);
        
              configuration.backup_path = Some(cache.add_empty_window_backup(&backup_folder, created_window.label()));
            }
    
            Ok(created_window)
          },
          Err(e) => {
            Err(Error::RwLock(e.to_string()))
          }
        }
      },
      Err(e) => {
        Err(Error::Tauri(e))
      }
    }
  }
}

fn open_folder_in_window<'a, R: Runtime, M: Manager<R>>(
  manager: &'a M,
  configuration: &OpenConfiguration,
  folder_to_open: Option<PathBuf>,
  force_new_window: bool,
  files_to_open: Option<FilesToOpen>,
  window_to_use: Option<String>,
) -> Result<Window<R>> {
  open_in_webview_window(
    manager,
    WindowOptions {
      folder: folder_to_open,
      initial_startup: configuration.initial_startup,
      force_new_window,
      force_new_tabbed_window: configuration.force_new_tabbed_window,
      files_to_open: files_to_open.unwrap_or(FilesToOpen::default()),
      window_to_use,
      ..Default::default()
    },
  )
}

fn open_in_empty_window<'a, R: Runtime, M: Manager<R>>(
  manager: &'a M,
  configuration: &OpenConfiguration,
  force_new_window: bool,
  files_to_open: FilesToOpen,
  empty_window_backup_info: Option<EmptyWindowBackupInfo>,
) -> Result<Window<R>> {
  let window_to_use = match &configuration.context_window_label {
    Some(label) => Some(label.to_string()),
    None => None
  };

  open_in_webview_window(
    manager,
    WindowOptions {
      initial_startup: configuration.initial_startup,
      force_new_window,
      force_new_tabbed_window: configuration.force_new_tabbed_window,
      files_to_open,
      window_to_use,
      empty_window_backup_info,
      ..Default::default()
    }
  )
}

fn is_single_folder<'a, R: Runtime, M: Manager<R>>(
  manager: &'a M,
  label: &str
) -> Result<bool> {
  let windows_state_cache = manager.state::<WindowsStateCache>();

  let res = match windows_state_cache.0.read() {
    Ok(cache) => {
      if let Some(window_state) = cache.get_item(label) {
        match &window_state.folder {
          Some(_uri) => Ok(true),
          None => Ok(false)
        }
      } else {
        Err(Error::WindowStateWithLabelNotFound(label.to_string()))
      }
    },
    Err(e) => {
      Err(Error::RwLock(e.to_string()))
    }
  };
  
  res
}

fn empty_files_to_open(files_to_open: &mut FilesToOpen) -> Result<()> {
  files_to_open.files_to_open_or_create.clear();
  files_to_open.files_to_open_or_create.shrink_to_fit();

  Ok(())
}

fn is_empty_files_to_open(files_to_open: &FilesToOpen) -> bool {
  files_to_open.files_to_open_or_create.is_empty()
}

// Implement window creation
fn open<'a, R: Runtime, M: Manager<R>>(
  manager: &'a M,
  configuration: &OpenConfiguration,
  mut folders_to_open: Vec<PathToOpen>,
  empty_to_restore: Vec<EmptyWindowBackupInfo>,
  mut empty_to_open: u8,
  mut files_to_open: FilesToOpen,
  folders_to_add: Vec<PathToOpen>,
) -> Result<Window<R>> {
  let mut used_windows: Vec<String> = Vec::new();
  let mut files_opened_in_window: Option<Window<R>> = None;
  
  // Settings can decide if files/folders open in new window or not
  let OpenOptions {
    mut open_folder_in_new_window,
    open_files_in_new_window,
  } = should_open_new_window(manager, configuration);

  // Handle folders to add by looking for the last active window (not on initial startup)
  if !configuration.initial_startup && folders_to_add.len() > 0 {
    if let Some(active_window) = get_last_active_window(manager) {
      add_folders_to_existing_window(&active_window, &folders_to_add)?;
      used_windows.push(active_window.label().to_string());
    }
  }

  // Handle files to open/diff or to create when we dont open a folder and we do not restore any
  // folder/untitled from hot-exit by trying to open them in the window that fits best
  let potential_new_windows_count: usize = folders_to_open.len() + empty_to_restore.len();
  if potential_new_windows_count == 0 {
    let file_to_check: Option<PathBuf> = match files_to_open.files_to_open_or_create.is_empty() {
      true => None,
      false => files_to_open.files_to_open_or_create.first().cloned(),
    };
  
    let mut window_to_use_for_files: Option<Window<R>> = None;
  
    if file_to_check.is_some() && !open_files_in_new_window {
      if configuration.context == OpenContext::Desktop
        || configuration.context == OpenContext::Cli
        || configuration.context == OpenContext::Dock
      {
        window_to_use_for_files = find_window_on_file(manager, file_to_check.as_ref().unwrap());
      }
  
      if !window_to_use_for_files.is_some() {
        window_to_use_for_files = get_last_active_window(manager);
      }
    }

    // We found a window to open the files in
    if let Some(window_to_use) = window_to_use_for_files {
      // Window is single folder
      let is_single_folder_res = is_single_folder(manager, window_to_use.label())?;

      if is_single_folder_res {
        folders_to_open.push(PathToOpen {
          folder: file_to_check,
          ..Default::default()
        });
      }
      // Window is empty
      else {
        match open_files_in_existing_window(
          configuration,
          &window_to_use,
          &files_to_open
        ) {
          Ok(_res) => {
            used_windows.push(window_to_use.label().to_string());
            files_opened_in_window = Some(window_to_use);
            empty_files_to_open(&mut files_to_open)
          },
          Err(e) => Err(e)
        }?;
      }
    }
    // Finally, if no window or folder is found, just open the files in an empty window
    else {
      let open_in_webview_window_res = open_in_webview_window(
        manager,
        WindowOptions {
          initial_startup: configuration.initial_startup,
          files_to_open: files_to_open.clone(),
          force_new_window: true,
          force_new_tabbed_window: configuration.force_new_tabbed_window,
          ..Default::default()
        },
      )?;
      
      used_windows.push(open_in_webview_window_res.label().to_string());
      files_opened_in_window = Some(open_in_webview_window_res);
      empty_files_to_open(&mut files_to_open)?;
    } 
  }

  // Handle folders to open (instructed and to restore)
  if folders_to_open.len() > 0 {

    // Check for existing instances
    let windows_on_folder_path = folders_to_open.iter()
    .filter_map(|folder_to_open| {
      find_window_on_folder(manager, folder_to_open.folder.as_ref())
    })
    .collect::<Vec<String>>();

    if windows_on_folder_path.len() > 0 {
      // Do open files
      if let Some(window_on_folder) = manager.get_window(&windows_on_folder_path[0]) {
        open_files_in_existing_window(configuration, &window_on_folder, &files_to_open)?;

        used_windows.push(window_on_folder.label().to_string());
        files_opened_in_window = Some(window_on_folder);
        empty_files_to_open(&mut files_to_open)?;
      }

      open_folder_in_new_window = true; // any other folders to open must open in new window then
    }

    // Open remaining ones
    for folder_to_open in folders_to_open {
      let window_already_opened = windows_on_folder_path.iter().find(|window| {
        if let Some(label) = find_window_on_folder(manager, folder_to_open.folder.as_ref()) {
          window.as_str().eq(&label)
        }
        else {
          false
        }
      });
      // ignore folders that are already open
      if window_already_opened.is_none() {
        // Do open folder
        let open_folder_in_window_res = open_folder_in_window(
          manager,
          configuration,
          folder_to_open.folder,
          open_folder_in_new_window,
          Some(files_to_open.clone()),
          None
        )?;

        used_windows.push(open_folder_in_window_res.label().to_string());
        files_opened_in_window = Some(open_folder_in_window_res);
        empty_files_to_open(&mut files_to_open)?;

        open_folder_in_new_window = true; // any other folders to open must open in new window then
      }
    }
  }

  // Handle empty to restore
  if empty_to_restore.len() > 0 {
    for empty_window_backup_info in &empty_to_restore {
      let empty_window_res = open_in_empty_window(
        manager,
        configuration,
        true,
        files_to_open.clone(),
        Some(empty_window_backup_info.clone())
      )?;

      used_windows.push(empty_window_res.label().to_string());
      files_opened_in_window = Some(empty_window_res);
      empty_files_to_open(&mut files_to_open)?;

      open_folder_in_new_window = true; // any other folders to open must open in new window then
    }
  }

  // Handle empty to open (only if no other window opened)
  if used_windows.len() == 0 || !is_empty_files_to_open(&files_to_open) {
    if !is_empty_files_to_open(&files_to_open) && empty_to_open == 0 {
      empty_to_open += 1;
    }

    for _i in 0..empty_to_open {
      // addUsedWindow(this.doOpenEmpty(openConfig, openFolderInNewWindow, remoteAuthority, filesToOpen), !!filesToOpen);
      let empty_window_res = open_in_empty_window(
        manager,
        configuration,
        open_folder_in_new_window,
        files_to_open.clone(),
        None
      )?;

      files_opened_in_window = Some(empty_window_res);
      empty_files_to_open(&mut files_to_open)?;

      open_folder_in_new_window = true; // any other folders to open must open in new window then
    }
  }

  match files_opened_in_window {
    Some(window) => Ok(window),
    None => Err(Error::Tauri(TauriError::CreateWindow))
  }
}

// API for window creation
pub struct WindowsAPI {}
impl WindowsAPI {
  pub fn get_focused_window<'a, R: Runtime, M: Manager<R>>(_manager: &'a M) -> Option<Window<R>> {
    None
  }
  
  pub fn get_last_active_window<'a, R: Runtime, M: Manager<R>>(
    manager: &'a M,
  ) -> Option<Window<R>> {
    get_last_active_window(manager)
  }

  pub fn open_window<'a, R: Runtime, M: Manager<R>>(
    manager: &'a M,
    configuration: OpenConfiguration
  ) -> Result<Window<R>> {
    let mut folders_to_open: Vec<PathToOpen> = Vec::new();
    let folders_to_add: Vec<PathToOpen> = Vec::new();
    let mut empty_windows_with_backups_to_restore: Vec<EmptyWindowBackupInfo> = Vec::new();
    let mut files_to_open: FilesToOpen = FilesToOpen {
      ..Default::default()
    };
    let mut empty_to_open: u8 = 0;

    // Identify things to open from open config
    let paths_to_open = get_paths_to_open(
      manager,
      &configuration.uris_to_open,
      &configuration.force_empty_window,
      &configuration.initial_startup,
    );

    for path_to_open in &paths_to_open {
      if path_to_open.folder.is_some() {
        folders_to_open.push(path_to_open.clone());
      } else if path_to_open.file.is_some() {
        let file = path_to_open.file.as_ref().unwrap();
        files_to_open
          .files_to_open_or_create
          .push(file.to_path_buf());
      } else if path_to_open.backup_path.is_some() {
        let backup = path_to_open.backup_path.as_ref().unwrap();
        empty_windows_with_backups_to_restore.push(EmptyWindowBackupInfo {
          backup_folder: Some(backup.to_path_buf()),
          ..Default::default()
        }); // get basename of path for folder name
      } else {
        empty_to_open += 1;
      }
    }

    // These are windows to restore because of hot-exit or from previous session (only performed once on startup!)
    if configuration.initial_startup {
      // Empty windows with backups are always restored
      let windows_backup_cache = manager.state::<WindowsBackupCache>();

      match windows_backup_cache.0.read() {
        Ok(cache) => {
          empty_windows_with_backups_to_restore
          .extend(cache.backups.empty_windows.iter().cloned());
        },
        Err(e) => {

        }
      };
    } else {
      empty_windows_with_backups_to_restore.clear();
    }

    // Open based on config
    let open_res = open(
      manager,
      &configuration,
      folders_to_open,
      empty_windows_with_backups_to_restore,
      empty_to_open,
      files_to_open,
      folders_to_add,
    );

    let windows_recents_cache = manager.state::<WindowsRecentsCache>();

    let res = match windows_recents_cache.0.write() {
      Ok(mut cache) => {
        let recents = paths_to_open.iter().filter_map(|path_to_open| {
          if path_to_open.folder.is_some() || path_to_open.file.is_some() {
            Some(path_to_open.clone())
          }
          else {
            None
          }
        }).collect::<Vec<PathToOpen>>();      
        
        cache.add_recents(recents);

        open_res
      },
      Err(e) => {
        Err(Error::RwLock(e.to_string()))
      }
    };

    res
  }
  
  pub fn open_empty_window<'a, R: Runtime, M: Manager<R>>(
    manager: &'a M,
    configuration: OpenConfiguration,
    options: WindowOptions
  ) -> Result<Window<R>> {
    let empty_configuration = OpenConfiguration {
      force_empty_window: true,
      force_reuse_window: options.force_reuse_window,
      force_new_window: !options.force_reuse_window,
      ..configuration
    };

    open(
      manager,
      &empty_configuration,
      Vec::new(),
      Vec::new(),
      0,
      FilesToOpen {
        ..Default::default()
      },
      Vec::new(),
    )

  }

  pub fn open_existing_window<'a, R: Runtime, M: Manager<R>>(
    _manager: &'a M,
    window: &Window<R>,
    _configuration: OpenConfiguration
  ) -> Result<()> {
    // Bring window to front
    window.set_focus().map_err(|e| Error::Tauri(e))?;

    Ok(())
  }

  pub fn send_to_focused() -> () {}
  pub fn send_to_all() -> () {}
}

#[cfg(test)]
mod tests {}
