mod application;
mod domain;
mod infrastructure;
mod presentation;

use tauri::Manager;

use infrastructure::db::connection::open_database;
use presentation::commands::{auth, export, notes, search, tags, timeline};
use presentation::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data directory");
            let db_path = app_data_dir.join("locked-calendar.db");
            let conn = open_database(&db_path).expect("failed to open database");
            app.manage(AppState::new(conn));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth::auth_is_initialized,
            auth::auth_status,
            auth::auth_setup,
            auth::auth_unlock,
            auth::auth_lock,
            auth::auth_touch_session,
            auth::auth_get_lock_timeout,
            auth::auth_set_lock_timeout,
            auth::auth_change_password,
            notes::notes_get_today,
            notes::notes_get_by_date,
            notes::notes_get_or_create,
            notes::notes_save,
            notes::notes_delete,
            notes::notes_toggle_favorite,
            notes::notes_list_month,
            search::search_notes,
            timeline::timeline_list,
            timeline::notes_on_this_day,
            tags::tags_list,
            export::export_notes_json,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
