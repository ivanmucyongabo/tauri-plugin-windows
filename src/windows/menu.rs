//! Menu.
//!
//! This module contains basic implemention of menu, submenu, and menu item types.
//! Provides builder methods for menus, submenus, and menu items.

use tauri::{
  CustomMenuItem,
  Menu as TauriMenu,
  MenuEntry,
  MenuItem,
  Submenu
};

use crate::event::{
  WINDOW_NEW_WINDOW_EVENT,
  WINDOW_OPEN_FILE_EVENT,
  WINDOW_OPEN_FOLDER_EVENT,
  WINDOW_CLOSE_WINDOW_EVENT,
  WINDOW_CLOSE_FILE_EVENT,
  WINDOW_CLOSE_FOLDER_EVENT
};

/// Window menu builder.
pub struct Menu {}

impl Menu {
  /// Creates a new custom menu item for new window.
  pub fn new_window<T: Into<String>>(title: T) -> CustomMenuItem {
    CustomMenuItem::new(WINDOW_NEW_WINDOW_EVENT, title).accelerator("CTRL+SHIFT+N")
  }

  /// Creates a new custom menu item for open file.
  pub fn open_file<T: Into<String>>(title: T) -> CustomMenuItem {
    CustomMenuItem::new(WINDOW_OPEN_FILE_EVENT, title).accelerator("CTRL+O")
  }

  /// Creates a new custom menu item for open folder.
  pub fn open_folder<T: Into<String>>(title: T) -> CustomMenuItem {
    CustomMenuItem::new(WINDOW_OPEN_FOLDER_EVENT, title).accelerator("CTRL+K+O")
  }
  
  /// Creates a new custom menu item for close window.
  pub fn close_window<T: Into<String>>(title: T) -> CustomMenuItem {
    CustomMenuItem::new(WINDOW_CLOSE_WINDOW_EVENT, title).accelerator("ALT+F4")
  }

  /// Creates a new custom menu item for close folder.
  pub fn close_folder<T: Into<String>>(title: T) -> CustomMenuItem {
    CustomMenuItem::new(WINDOW_CLOSE_FOLDER_EVENT, title).accelerator("CTRL+F")
  }

  /// Creates a new custom menu item for close file.
  pub fn close_file<T: Into<String>>(title: T) -> CustomMenuItem {
    CustomMenuItem::new(WINDOW_CLOSE_FILE_EVENT, title).accelerator("CTRL+F4")
  }

  /// Creates a menu filled with default menu items and submenus.
  /// 
  /// ## Platform-specific:
  ///
  /// - **Windows**:
  ///   - File
  ///     - New Window
  ///     - Open File
  ///     - Open Folder
  ///     - Close File
  ///     - Close File
  ///     - Close Folder
  ///     - Close Window
  pub fn default() -> TauriMenu {
    TauriMenu::with_items([Menu::as_submenu("File").into()])
  }

  /// Creates a menu filled with default menu items and submenu with user provided title.
  /// 
  /// # Examples
  /// ```
  /// # use tauri_plugin_windows::windows::{Menu};
  /// Menu::new("menu_title");
  /// ```
  /// 
  /// ## Platform-specific:
  ///
  /// - **Windows**:
  ///   - File
  ///     - New Window
  ///     - Open File
  ///     - Open Folder
  ///     - Close File
  ///     - Close File
  ///     - Close Folder
  ///     - Close Window
  pub fn new<S: Into<String>>(title: S) -> TauriMenu {
    TauriMenu::with_items([Menu::as_submenu(title).into()])
  }

  /// Creates menu with default menu items.
  /// 
  /// ## Platform-specific:
  ///
  /// - **Windows**:
  ///   - New Window
  ///   - Open File
  ///   - Open Folder
  ///   - Close File
  ///   - Close File
  ///   - Close Folder
  ///   - Close Window
  pub fn as_menu() -> TauriMenu {
    TauriMenu::with_items([
      Menu::new_window("New Window").into(),
      Menu::open_file("Open File").into(),
      Menu::open_folder("Open Folder").into(),
      Menu::close_file("Close File").into(),
      Menu::close_folder("Close Folder").into(),
      Menu::close_window("Close Window").into(),
      MenuItem::Separator.into(),
      MenuItem::Quit.into(),
    ])
  }

  /// Creates a submenu filled with default menu items, user provided title.
  /// 
  /// # Examples
  /// ```
  /// # use tauri_plugin_windows::windows::{Menu};
  /// Menu::as_submenu("menu_title");
  /// ```
  /// 
  /// ## Platform-specific:
  ///
  /// - **Windows**:
  ///   - File
  ///     - New Window
  ///     - Open File
  ///     - Open Folder
  ///     - Close File
  ///     - Close File
  ///     - Close Folder
  ///     - Close Window  
  pub fn as_submenu<S: Into<String>>(title: S) -> Submenu {
    Submenu::new(title, Menu::as_menu())
  }

  /// Creates vector of default menu items.
  pub fn menu_items() -> Vec<MenuEntry> {
    vec![
      Menu::new_window("New Window").into(),
      Menu::open_file("Open File").into(),
      Menu::open_folder("Open Folder").into(),
      Menu::close_file("Close File").into(),
      Menu::close_folder("Close Folder").into(),
      Menu::close_window("Close Window").into(),
      MenuItem::Separator.into(),
      MenuItem::Quit.into(),
    ]
  }
}
