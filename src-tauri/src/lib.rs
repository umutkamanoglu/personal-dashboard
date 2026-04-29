mod features;

use features::tmdb::*;

#[tauri::command]
fn get_system_stats() -> features::monitor::SystemData {
    features::monitor::get_full_system_info()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_system_stats,
            get_discover_movie,
            get_discover_series,
            get_item_details,
            search_all,
            get_genres
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
