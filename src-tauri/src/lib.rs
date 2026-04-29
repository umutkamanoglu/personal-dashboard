mod features;

use features::media::{get_current_media_info, process_media_command, MediaInfo};
use features::tmdb::*;

#[tauri::command]
fn get_system_stats() -> features::monitor::SystemData {
    features::monitor::get_full_system_info()
}

// ARTIK BURASI ASYNC OLMALI
#[tauri::command]
async fn get_active_media() -> Option<MediaInfo> {
    get_current_media_info().await // .await eklemeyi unutmayın
}

#[tauri::command]
async fn send_media_command(command: String) {
    process_media_command(command).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_system_stats,
            get_discover_movie,
            get_discover_series,
            get_item_details,
            search_all,
            get_genres,
            get_active_media,
            send_media_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
