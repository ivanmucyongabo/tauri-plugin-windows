//! Window.
//!
//! This module contains basic methods and traits for managing window state. 
//! Provides traits that can used to extend window.
//! Extends window.

use std::{
  collections::{HashMap, hash_map::DefaultHasher},
  fs::File,
  path::{PathBuf, Path},
  result::Result as StdResult,
  sync::{
    atomic::{AtomicUsize, Ordering},
    RwLock
  },
  time::{Duration, SystemTime, UNIX_EPOCH}, io::Write, hash::{Hasher, Hash},
};

use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};

use tauri::{
  api::{
    file::read_string,
    path::{data_dir}
  },
  window::Monitor, Manager, PhysicalPosition,
  PhysicalSize, Runtime, Window,
};

use crate::error::Error;

use super::PathToOpen;

type Result<T> = StdResult<T, Error>;

static COUNTER: AtomicUsize = AtomicUsize::new(1);
const STATE_FILENAME: &str = ".windows_state_session";
const BACKUP_FILENAME: &str = ".windows_backup_session";
const RECENTS_FILENAME: &str = ".windows_recents_session";
const MAX_TOTAL_RECENT_ENTRIES: u16 = 500;
const RECENTLY_OPENED_STORAGE_KEY: &str = "history.recently_opened_paths_list";

/// Window bounds rectangle.
/// 
/// A rectangle representing the window.
pub struct WindowBounds {
  pub x: i32,
  pub y: i32,
  pub height: u32,
  pub width: u32,
}

impl Default for WindowBounds {
  fn default() -> Self {
    Self {
      x: 0,
      y: 0,
      height: 0,
      width: 0,
    }
  }
}

/// Window screen mode.
/// 
/// A flag representing the screen style for the window.
#[derive(Clone, Deserialize, PartialEq, Serialize)]
pub enum WindowMode {
  Maximized,
  Normal,
  Minimized,
  Fullscreen,
}

impl Default for WindowMode {
  fn default() -> Self {
    WindowMode::Normal
  }
}

/// Ready state for winodw.
/// 
/// A flag for whether the window can currently handle events.
#[derive(Clone, Deserialize, PartialEq, Serialize)]
pub enum ReadyState {
  None,
  Navigating,
  Ready,
}

impl Default for ReadyState {
  fn default() -> Self {
    ReadyState::None
  }
}

/// Window configuration cached during runtime.
/// 
/// Contains info that will be backed up.
#[derive(Clone, Default, Deserialize, PartialEq, Serialize)]
pub struct WindowConfiguration {
  pub folder: Option<PathBuf>,
  pub files_to_open_or_create: Vec<PathBuf>,
  pub full_screen: bool,
  pub maximized: bool,
  pub cache_path: Option<PathBuf>,
  pub backup_path: Option<PathBuf>,
	pub home_dir: Option<PathBuf>,
	pub tmp_dir: Option<PathBuf>,
	pub user_data_dir: Option<PathBuf>,
  pub is_initial_startup: bool,
}

/// Window state cached during runtime
#[derive(Clone, Default, Deserialize, PartialEq, Serialize)]
pub struct WindowState {
  pub configuration: WindowConfiguration,
  pub mode: WindowMode,
  pub display: u32,
  pub id: u32,
  pub last_focus_time: Duration,
  pub ready_state: ReadyState,
  pub backup_path: Option<PathBuf>,
  pub folder: Option<PathBuf>,
}

#[derive(Clone, Default, Deserialize, PartialEq, Serialize)]
pub struct LastActiveWindow {
  pub label: String,
  pub state: WindowState
}

#[derive(Clone, Default, Deserialize, PartialEq, Serialize)]
pub struct WindowsState {
  pub opened_windows: HashMap<String, WindowState>,
  pub last_active_window: Option<LastActiveWindow>,
  pub focused_window: Option<String>,
  pub was_restarted: bool
}

/// Managed state for cache in memory cache of window states during runtime.
/// 
/// Provides in memory cache, and file back up
#[derive(Clone, Deserialize, Serialize)]
pub struct InnerWindowsStateCache{
  pub storage_path: PathBuf,
  pub last_saved_storage_contents: String,
  pub storage: WindowsState
}

impl InnerWindowsStateCache {

  pub fn new(file: &Path) -> Self {
    match read_string(file) {
      Ok(contents) => {
        let storage = match from_str(&contents) {
          Ok(deserialized) => deserialized,
          Err(e) => {
            WindowsState::default()
          }
        };

        InnerWindowsStateCache {
          storage_path: file.to_path_buf(), 
          last_saved_storage_contents: contents,
          storage: storage
        }
      },
      Err(e) => {
        InnerWindowsStateCache {
          storage_path: PathBuf::new(), 
          last_saved_storage_contents: "".to_string(),
          storage: WindowsState::default()
        }        
      }
    }
  }  

  pub fn storage(&self) -> &WindowsState {
    &self.storage
  }  

  pub fn state(&self) -> &WindowsState {
    &self.storage
  }

  pub fn state_mut(&mut self) -> &mut WindowsState {
    &mut self.storage
  }

  pub fn get_state(&self) -> WindowState {WindowState::default()}
  pub fn get_state_mut(&self) -> WindowState {WindowState::default()}
  pub fn set_state(&self) -> Result<()> {Ok(())}
  pub fn handle_destroyed_window(&mut self, label: &str) {}
  pub fn handle_focused_window(&mut self, label: &str, focus: &bool) {}
  pub fn handle_close_window(&mut self, label: &str) {}

  pub fn get_item(&self, key: &str) -> Option<WindowState> {
    match self.storage.opened_windows.get(key) {
      Some(window) => Some(window.clone()),
      None => None
    }
  }

  pub fn set_item(&mut self, key: &str, data: WindowState) -> Result<()> {
    match self.storage.opened_windows.insert(key.to_string(), data) {
      Some(state) => {
        self.save()
      },
      None => Err(Error::WindowStateWithLabelNotFound(key.to_string()))
    }
  }
  
  pub fn set_items<I>(&mut self, items: I) -> Result<()> 
  where
    I: Iterator<Item = (String, WindowState)>
  {
    let mut save = false;

    for (key, data) in items {
      // Shortcut for data that did not change
      if let Some(stored_data) = self.storage.opened_windows.get(&key) {
        if *stored_data == data {
          continue;
        }
      }
      // Otherwise add an item
      else {
        match self.storage.opened_windows.insert(key, data) {
          Some(state) => {
            save = true;
          },
          None => {}
        };
      }
    }

    if !save {
      return Ok(())
    }else {
      self.save()
    }
    
  }
  
  pub fn remove_item(&mut self, key: &str) -> Result<()> {
    match self.storage.opened_windows.remove(key) {
      Some(state) => Ok(()),
      None => Err(Error::WindowStateWithLabelNotFound(key.to_string()))
    }
  }
  
  pub fn close(&mut self) -> Result<()> {
    self.save()
  }
  
  fn save(&mut self) -> Result<()> {
    let serialized_database = to_string(&self.storage).unwrap_or("".to_string());

		// Return early if the database has not changed
    if self.last_saved_storage_contents.eq(&serialized_database) {
      Ok(())
    }
    // Write to disk
    else {
      match File::create(&self.storage_path) {
        Ok(mut file) => {
          file.write_all(serialized_database.as_bytes())?;
          self.last_saved_storage_contents = serialized_database;

          Ok(())
        },
        Err(e) => {
          Ok(())
        }
      }
    }
  }
}

impl Default for InnerWindowsStateCache {
  fn default() -> Self {
    if let Some(dir) = data_dir() {
      match read_string(dir.join(STATE_FILENAME)) {
        Ok(contents) => {
          let storage = match from_str(&contents) {
            Ok(deserialized) => deserialized,
            Err(e) => {
              WindowsState::default()
            }
          };
  
          InnerWindowsStateCache {
            storage_path: dir.join(STATE_FILENAME), 
            last_saved_storage_contents: contents,
            storage: storage
          }
        },
        Err(e) => {
          InnerWindowsStateCache {
            storage_path: PathBuf::new(), 
            last_saved_storage_contents: "".to_string(),
            storage: WindowsState::default()
          }        
        }
      }
    }else {
      InnerWindowsStateCache {
        storage_path: PathBuf::new(), 
        last_saved_storage_contents: "".to_string(),
        storage: WindowsState::default(),
      }
    }
  }
}

/// Managed state for window states.
#[derive(Default, Deserialize, Serialize)]
pub struct WindowsStateCache(pub RwLock<InnerWindowsStateCache>);

/// Folder backup info.
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct FolderBackupInfo {
  pub window: String,
  pub folder: Option<PathBuf>
}

/// Empty window backup info.
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct EmptyWindowBackupInfo {
  pub window: String,
  pub backup_folder: Option<PathBuf>,
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct WindowsBackup {
  pub folders: Vec<FolderBackupInfo>,
	pub empty_windows: Vec<EmptyWindowBackupInfo>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct InnerWindowsBackupCache {
  pub backup_path: PathBuf,
  pub backups: WindowsBackup
}

impl InnerWindowsBackupCache {
  pub fn new(file: &Path) -> Self {
    match read_string(file) {
      Ok(contents) => {
        let backups = match from_str(&contents) {
          Ok(deserialized) => deserialized,
          Err(e) => {
            WindowsBackup::default()
          }
        };

        InnerWindowsBackupCache {
          backup_path: file.to_path_buf(),
          backups
        }
      },
      Err(e) => {
        InnerWindowsBackupCache {
          backup_path: PathBuf::new(),
          backups: WindowsBackup::default()
        }        
      }
    }
  }

  pub fn add_folder_backup(&mut self, folder: &PathBuf, window: &str) -> PathBuf {
		if !self.backups.folders.iter().any(
      |info| info.folder.as_ref().and_then(
        |backup_folder| if backup_folder.eq(folder) {Some(true)} else {None}
      ).is_some()
    ) {
			self.backups.folders.push(FolderBackupInfo {
        window: window.to_string(),
        folder: Some(folder.clone())
      });
			self.save();
		}

	  self.get_backup_path(&self.get_folder_hash(folder))
  }

  pub fn add_empty_window_backup(&mut self, backup_folder_candidate: &Option<PathBuf>, window: &str) -> PathBuf {
		// Generate a new folder if this is a new empty workspace
    let backup_folder = match backup_folder_candidate {
      Some(folder) => folder.clone(),
      None => self.get_random_empty_window_id()
    };

		if !self.backups.empty_windows.iter().any(
      |empty_window| empty_window.backup_folder.as_ref().and_then(
        |folder| if folder.eq(&backup_folder) {Some(true)} else {None}
      ).is_some()
    ) {
      let folder = Some(backup_folder.clone());

			self.backups.empty_windows.push(EmptyWindowBackupInfo {
        backup_folder: folder,
        window: window.to_string()
      });
			self.save();
		}

		self.get_backup_path(&backup_folder)
  }
  
  pub fn get_random_empty_window_id(&self) -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed).to_string();

    PathBuf::from(id)
  }
  
  pub fn get_folder_hash(&self, folder: &PathBuf) -> PathBuf {
    let mut s = DefaultHasher::new();
    folder.hash(&mut s);
    PathBuf::from(s.finish().to_string())
  }

  pub fn get_backup_path(&self, folder: &PathBuf) -> PathBuf {
    self.backup_path.join(folder)
  }
  
  fn save(&self) {}
}

impl Default for InnerWindowsBackupCache {
  fn default() -> Self {
    if let Some(dir) = data_dir() {
      match read_string(dir.join(BACKUP_FILENAME)) {
        Ok(contents) => {
          let backups = match from_str(&contents) {
            Ok(deserialized) => deserialized,
            Err(e) => {
              WindowsBackup::default()
            }
          };
  
          InnerWindowsBackupCache {
            backup_path: dir,
            backups
          }
        },
        Err(e) => {
          InnerWindowsBackupCache {
            backup_path: dir,
            backups: WindowsBackup::default()
          }        
        }
      }
    }else {
      InnerWindowsBackupCache {
        backup_path: PathBuf::new(),
        backups: WindowsBackup::default()
      } 
    }
  }
}

/// Managed state for window backup locations.
#[derive(Default)]
pub struct WindowsBackupCache(pub RwLock<InnerWindowsBackupCache>);

#[derive(Default)]
pub struct RecentPath {
  pub label: Option<String>,
  pub folder: Option<PathBuf>,
  pub file: Option<PathBuf>,
  pub window: Option<String>
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct RecentFile {
  pub label: String,
  pub file: PathBuf,
  pub window: String
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct RecentFolder {
  pub label: String,
  pub folder: PathBuf,
  pub window: String
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct RecentlyOpened {
  files: Vec<RecentFile>,
  folders: Vec<RecentFolder>
}

#[derive(Clone, Deserialize, Serialize)]
pub struct InnerWindowsRecentsCache {
  pub recents_path: PathBuf,
  last_saved_recents_contents: String,
  pub recents: RecentlyOpened
}

impl InnerWindowsRecentsCache {
  pub fn new(file: &Path) -> Self {
    match read_string(file) {
      Ok(contents) => {
        let recents = match from_str(&contents) {
          Ok(deserialized) => deserialized,
          Err(e) => {
            RecentlyOpened::default()
          }
        };

        InnerWindowsRecentsCache {
          last_saved_recents_contents: contents,
          recents_path: file.to_path_buf(),
          recents
        }
      },
      Err(e) => {
        InnerWindowsRecentsCache {
          last_saved_recents_contents: "".to_string(),
          recents_path: PathBuf::new(),
          recents: RecentlyOpened::default()
        }        
      }
    }  
  }
  
  pub fn add_recents(&mut self, recents: Vec<PathToOpen>) -> Result<()> {
    for recent in recents {
      if recent.folder.is_some() {
        self.recents.folders.push(RecentFolder {
          label: recent.label.unwrap(),
          folder: recent.folder.unwrap(),
          window: recent.window.unwrap()
        });
      }
      else {
        self.recents.files.push(RecentFile {
          label: recent.label.unwrap(),
          file: recent.file.unwrap(),
          window: recent.window.unwrap()
        });
      }
    }

		if self.recents.folders.len() > MAX_TOTAL_RECENT_ENTRIES.into() {
			self.recents.folders.truncate(MAX_TOTAL_RECENT_ENTRIES.into());
		}

		if self.recents.files.len() > MAX_TOTAL_RECENT_ENTRIES.into() {
			self.recents.files.truncate(MAX_TOTAL_RECENT_ENTRIES.into());
		}

		self.save()
  }

  pub fn add_recent(&mut self, path: PathToOpen) {}

  pub fn remove_recent(&mut self) {

  }
  
  pub fn clear(&mut self) {
    self.recents.folders.clear();
    self.recents.folders.shrink_to_fit();
    self.recents.files.clear();
    self.recents.files.shrink_to_fit();
  }
  
  pub fn get_recent(&self) {

  }

  fn save(&mut self) -> Result<()> {
    let serialized_database = to_string(&self.recents).unwrap_or("".to_string());

		// Return early if the database has not changed
    if self.last_saved_recents_contents.eq(&serialized_database) {
      Ok(())
    }
    // Write to disk
    else {
      match File::create(&self.recents_path) {
        Ok(mut file) => {
          file.write_all(serialized_database.as_bytes())?;
          self.last_saved_recents_contents = serialized_database;

          Ok(())
        },
        Err(e) => {
          Ok(())
        }
      }
    }
  }
}

impl Default for InnerWindowsRecentsCache {
  fn default() -> Self {
    if let Some(dir) = data_dir() {
      match read_string(dir.join(BACKUP_FILENAME)) {
        Ok(contents) => {
          let recents = match from_str(&contents) {
            Ok(deserialized) => deserialized,
            Err(e) => {
              RecentlyOpened::default()
            }
          };
  
          InnerWindowsRecentsCache {
            last_saved_recents_contents: contents,
            recents_path: dir.join(BACKUP_FILENAME),
            recents: recents
          }
        },
        Err(e) => {
          InnerWindowsRecentsCache {
            last_saved_recents_contents: "".to_string(),
            recents_path: dir.join(BACKUP_FILENAME),
            recents: RecentlyOpened::default()
          }        
        }
      }
    }else {
      InnerWindowsRecentsCache {
        last_saved_recents_contents: "".to_string(),
        recents_path: PathBuf::new(),
        recents: RecentlyOpened::default()
      } 
    }
  }
}

#[derive(Default)]
pub struct WindowsRecentsCache(pub RwLock<InnerWindowsRecentsCache>);

/// Trait for [`WindowBounds`] helpers.
/// 
/// [`WindowBounds`]: WindowBounds
pub trait WindowBoundsTrait {
  /// Check whether window bounds are empty.
  /// 
  /// If the bounds doesn't have width or height it is considered empty.
  fn is_empty(&self) -> bool;

  /// Get window bounds for window
  fn get_bounds(&self) -> WindowBounds;

  /// Intersect window bounds with provided struct.
  /// 
  /// The provided struct must implement this trait.
  fn intersect(&self, window: impl WindowBoundsTrait);
}

/// Trait for ['WindowState'] helpers.
/// 
/// ['WindowState']: WindowState
pub trait WindowStateTrait {
  /// Get last focus time for this window.
  fn last_focus_time(&self) -> Result<Duration>;

  /// Update the last focus time for this window.
  /// 
  /// Sets the last focus time to the Duration at time of call.
  fn set_last_focus_time(&self) -> Result<()>;

  /// Update window state.
  /// 
  /// Does not backup before updating.
  fn set_window_state(&self, new_state: WindowState) -> Result<()>;

  /// Destroy window state.
  /// 
  /// Does not save before destroying.
  fn destroy_window_state(&self) -> Result<()>;
}

/// Trait for getting resource data from state
pub trait WindowFilesTrait {
  /// Get file path if available on the window state.
  fn file_uri(&self) -> Option<PathBuf>;

  /// Get folder path if available on the window state.
  fn folder_uri(&self) -> Option<PathBuf>;
}

/// Trait that provide extension to window.
/// 
/// Provides methods for asynchronous events, reload/reopen, and event handling.
pub trait WindowTrait {
  /// Load file or url into webview
  fn load(&self);

  /// Reload window, with cached up state.
  fn reload(&self);

  /// Reopen window, with backed up state.
  fn reopen(&self);

  /// Register callbacks in queue.
  /// 
  /// Puts call back into queue to be called in order of registration when window state is ready.
  fn register_listeners(&self);

  /// Update ready state for window.
  /// 
  /// Allows the window to start processing specific events.
  fn set_ready(&self) -> Result<()>;

  /// Ran when window is ready.
  fn ready(&self);

  /// Get window ready state.
  ///
  /// Returns false if not ready for event handling, and vice versa.
  fn is_ready(&self) -> bool;

  fn send_when_ready(&self);
  fn send(&self);


  fn handle_title_doublclick(&self);

  fn destroy(&self) -> Result<()>;
}

fn get_working_area(monitor: &Monitor) {
  todo!();
}
fn get_monitor_matching(window: &Window) {
  todo!();
}
fn get_monitor_nearest_point(window: &Window) {
  todo!();
}
fn find_monitor_nearest_point(monitors: &Vec<Monitor>, window: &Window) {
  todo!();
}
fn find_monitor_containing_point(monitors: &Vec<Monitor>, window: &Window) {
  todo!();
}
fn find_monitor_with_biggest_intersection(monitors: &Vec<Monitor>, window: &Window) {
  todo!();
}

fn intersect_windows(a: impl WindowBoundsTrait, b: impl WindowBoundsTrait) {
  let result = a.intersect(b);
  return result;
}
fn ray_intersects_segment() {
  todo!();
}

impl<R: Runtime> WindowBoundsTrait for Window<R> {
  fn is_empty(&self) -> bool {
    let bounds = self.get_bounds();
    return !(bounds.width != 0) || !(bounds.height != 0);
  }
  fn get_bounds(&self) -> WindowBounds {
    let outer_position = self
      .outer_position()
      .unwrap_or(PhysicalPosition { x: 0, y: 0 });
    let outer_size = self.outer_size().unwrap_or(PhysicalSize {
      height: 0,
      width: 0,
    });

    return WindowBounds {
      x: outer_position.x,
      y: outer_position.y,
      height: outer_size.height,
      width: outer_size.width,
    };
  }
  fn intersect(&self, _window: impl WindowBoundsTrait) -> () {}
}

impl<R: Runtime> WindowStateTrait for Window<R> {
  fn last_focus_time(&self) -> Result<Duration> {
    let window_states_cache = self.state::<WindowsStateCache>();

    let cache = window_states_cache.0.read()
    .map_err(|e| Error::RwLock(e.to_string()))?;

    match cache.get_item(self.label()) {
      Some(state) => Ok(state.last_focus_time),
      None => Ok(Duration::new(0, 0)),
    }
  }
  
  fn set_last_focus_time(&self) -> Result<()> {
    let window_states_cache = self.state::<WindowsStateCache>();

    let mut cache = window_states_cache.0.write()
    .map_err(|e| Error::RwLock(e.to_string()))?;

    match cache.storage.opened_windows.get_mut(self.label()) {
      Some(state) => {
        let start = SystemTime::now();

        state.last_focus_time = start
          .duration_since(UNIX_EPOCH)
          .expect("Time went backwards");

        Ok(())
      }
      None => Err(Error::WindowStateWithLabelNotFound(
        self.label().to_string(),
      )),
    }
  }

  fn set_window_state(&self, new_state: WindowState) -> Result<()> {
    let window_states_cache = self.state::<WindowsStateCache>();

    let mut cache = window_states_cache.0.write()
    .map_err(|e| Error::RwLock(e.to_string()))?;

    cache.set_item(self.label(), WindowState { ..new_state })
  }
  
  fn destroy_window_state(&self) -> Result<()> {
    let window_states_cache = self.state::<WindowsStateCache>();

    let mut cache = window_states_cache.0.write()
    .map_err(|e| Error::RwLock(e.to_string()))?;

    cache.remove_item(self.label())
  }
}

impl<R: Runtime> WindowFilesTrait for Window<R> {
  fn file_uri(&self) -> Option<PathBuf> {
    None
  }
  
  fn folder_uri(&self) -> Option<PathBuf> {
    None
  }
}

impl<R: Runtime> WindowTrait for Window<R> {
  fn load(&self) {}
  
  fn reload(&self) {}
  
  fn reopen(&self) {}

  fn register_listeners(&self) {}

  fn set_ready(&self) -> Result<()> {
    let window_states_cache = self.state::<WindowsStateCache>();

    let mut cache = window_states_cache.0.write()
    .map_err(|e| Error::RwLock(e.to_string()))?;

    match cache.storage.opened_windows.get_mut(self.label()) {
      Some(state) => {
        state.ready_state = ReadyState::Ready;
        Ok(())
      }
      None => Err(Error::WindowStateWithLabelNotFound(
        self.label().to_string(),
      )),
    }
  }
  
  fn ready(&self) {}
  
  fn is_ready(&self) -> bool {
    let window_states_cache = self.state::<WindowsStateCache>();

    let res = match window_states_cache.0.read() {
      Ok(cache) => {
        match cache.storage.opened_windows.get(self.label()) {
          Some(state) => state.ready_state == ReadyState::Ready,
          None => false,
        }
      },
      Err(e) => {
        false
      }
    };
    
    res
  }
  
  fn send_when_ready(&self) {}
  
  fn send(&self) {}

  fn handle_title_doublclick(&self) {}
  
  fn destroy(&self) -> Result<()> {
    self.destroy_window_state()?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {}
