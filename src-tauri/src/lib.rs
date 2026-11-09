mod appearance;
mod process;
mod tools;
mod ytdlp;

use std::path::Path;
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_window_state::StateFlags;

/// Opens a downloaded file with its default app (double-click in the list),
/// or a folder in Explorer.
#[tauri::command]
fn open_path(app: AppHandle, path: String) -> Result<(), String> {
    if !Path::new(&path).exists() {
        // A code, not a message: the UI shows it in the user's language.
        return Err("NOT_FOUND".into());
    }
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        // Remember size/position/maximized like a native app. Visibility is
        // excluded: the window starts hidden and the frontend shows it once
        // the first frame is painted (no white flash).
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(StateFlags::SIZE | StateFlags::POSITION | StateFlags::MAXIMIZED)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .manage(ytdlp::Jobs::default())
        .setup(|app| {
            let window = app
                .get_webview_window("main")
                .expect("main window is defined in tauri.conf.json");

            // Apply Mica + read the accent color off the main thread.
            let w = window.clone();
            thread::spawn(move || {
                appearance::init(&w);
            });

            // Safety net: show the window even if the frontend failed to load.
            thread::spawn(move || {
                thread::sleep(Duration::from_secs(4));
                let _ = window.show();
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            appearance::appearance,
            tools::tool_status,
            tools::install_ytdlp,
            tools::update_ytdlp,
            tools::install_ffmpeg,
            ytdlp::fetch_info,
            ytdlp::start_download,
            ytdlp::cancel_download,
            open_path,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|handle, event| {
        // Don't leave yt-dlp/ffmpeg running in the background after quitting.
        if let tauri::RunEvent::Exit = event {
            handle.state::<ytdlp::Jobs>().kill_all();
        }
    });
}
