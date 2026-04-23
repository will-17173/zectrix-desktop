mod api;
mod commands;
mod models;
mod state;
mod storage;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state::AppState::new().expect("app state"))
        .invoke_handler(tauri::generate_handler![
            commands::config::load_bootstrap_state,
            commands::config::list_api_keys,
            commands::config::add_api_key,
            commands::config::remove_api_key,
            commands::config::sync_all,
            commands::devices::add_device_cache,
            commands::devices::remove_device_cache,
            commands::devices::list_device_cache,
            commands::todos::create_local_todo,
            commands::todos::toggle_todo_status,
            commands::todos::delete_local_todo,
            commands::todos::update_local_todo,
            commands::todos::push_todo_to_device,
            commands::templates::create_text_template,
            commands::templates::push_text_template,
            commands::templates::push_text,
            commands::images::render_image_preview,
            commands::images::save_image_template,
            commands::images::get_image_thumbnail,
            commands::images::push_image_template,
            commands::images::delete_image_template,
            commands::images::push_sketch,
        commands::updater::check_for_update,
            commands::updater::get_current_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
