//! Event.
//!
//! This module contains basic functions and variables for.

/// Event label for new window.
/// Window specific event or Menu specific event.
pub const WINDOW_NEW_WINDOW_EVENT: &str = "windows://new_window";

/// Event label for close window.
/// Window specific event or Menu specific event.
pub const WINDOW_CLOSE_WINDOW_EVENT: &str = "windows://close_window";

/// Event label for open file.
/// Window specific event or Menu specific event.
pub const WINDOW_OPEN_FILE_EVENT: &str = "windows://open_file";

/// Event label for open files.
/// Global event
pub const WINDOW_OPEN_FILES_EVENT: &str = "windows://open_files";

/// Event label for add folder.
/// Global event
pub const WINDOW_ADD_FOLDERS_EVENT: &str = "windows://add_folders";

/// Event label for open folder.
/// Window specific event or Menu specific event.
pub const WINDOW_OPEN_FOLDER_EVENT: &str = "windows://open_folder";

/// Event label for close file.
/// Window specific event or Menu specific event.
pub const WINDOW_CLOSE_FILE_EVENT: &str = "windows://close_file";

/// Event label for close folder.
/// Window specific event or Menu specific event.
pub const WINDOW_CLOSE_FOLDER_EVENT: &str = "windows://close_folder";