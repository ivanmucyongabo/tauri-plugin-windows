#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri_plugin_windows::{
  TauriWindows,
  windows::{
    Menu,
    OpenConfiguration,
    OpenContext,
    WindowOpenable,
    WindowsService,
  }
};

use tauri::{
  api::dialog::FileDialogBuilder,
  generate_context,
  Builder,
  CustomMenuItem,
  Manager,
  Menu as TauriMenu,
  Submenu
};

fn main() {
  let context = generate_context!();

  let open_file = CustomMenuItem::new("open_file", "Open File");
  let open_folder = CustomMenuItem::new("open_folder", "Open Folder");
  let submenu = Submenu::new(
    "File",
    TauriMenu::with_items([
      Menu::new_window("New Window").into(),
      open_file.into(),
      open_folder.into(),
      Menu::close_file("Close File").into(),
      Menu::close_folder("Close Folder").into(),
      Menu::close_window("Close Window").into(),
    ])
  );
  let menu = TauriMenu::new().add_submenu(submenu);

  Builder::default()
    .menu(if cfg!(target_os = "macos") {
      TauriMenu::os_default(&context.package_info().name)
    } else {
      menu
    })
    .on_menu_event(|event| {
      match event.menu_item_id() {
        "open_file" => {
          FileDialogBuilder::new().pick_files(move |file_paths| {
            // do something with the optional file paths here
            // the file paths is `None` if the user closed the dialog
            if let Some(paths_to_open) = file_paths {
              // pub uris_to_open: Option<Vec<WindowOpenable>>
              // pub struct WindowOpenable {
              //   pub folder_uri: Option<PathBuf>,
              //   pub file_uri: Option<PathBuf>,
              // }
              let uris_to_open = paths_to_open.iter()
              .map(|path| WindowOpenable { file_uri: Some(path.to_path_buf()), ..Default::default() })
              .collect::<Vec<WindowOpenable>>();

              match WindowsService::open_window(
                &event.window().app_handle(),
                OpenConfiguration {
                  context: OpenContext::Dialog,
                  uris_to_open: Some(uris_to_open),
                  prefer_new_window: true,
                  ..Default::default()
                }) {
                Ok(_res) => {}
                Err(e) => {
                  eprintln!("Error: {:?}", e);
                },
              };              
            }
          })
        }
        "open_folder" => {
          FileDialogBuilder::new().pick_folders(move |folder_paths| {
            // do something with the optional folder path here
            // the folder path is `None` if the user closed the dialog
            if let Some(paths_to_open) = folder_paths {
              let uris_to_open = paths_to_open.iter()
              .map(|path| WindowOpenable { folder_uri: Some(path.to_path_buf()), ..Default::default() })
              .collect::<Vec<WindowOpenable>>();

              match WindowsService::open_window(
                &event.window().app_handle(),
                OpenConfiguration {
                  context: OpenContext::Dialog,
                  uris_to_open: Some(uris_to_open),
                  prefer_new_window: true,
                  ..Default::default()
                }) {
                Ok(_res) => {}
                Err(e) => {
                  eprintln!("Error: {:?}", e);
                },
              };              
            }            
          })
        }
        _ => {}
      }
    })    
    .plugin(TauriWindows::default())
    .run(context)
    .expect("failed to run app");
}
