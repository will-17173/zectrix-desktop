mod api;
mod builtin_plugins;
mod commands;
mod models;
pub mod plugin_output;
pub mod plugin_runtime;
pub mod plugin_tasks;
mod state;
mod stock_quote;
mod storage;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state::AppState::new().expect("app state"))
        .setup(|app| {
            // Stop all running tasks on startup
            let state = app.state::<state::AppState>();
            if let Err(e) = state.stop_all_image_loop_tasks_on_boot() {
                eprintln!("[startup] failed to stop image loop tasks: {e}");
            }
            if let Err(e) = state.stop_stock_push_task_on_boot() {
                eprintln!("[startup] failed to stop stock push task: {e}");
            }
            if let Err(e) = state.stop_all_plugin_loop_tasks_on_boot() {
                eprintln!("[startup] failed to stop plugin loop tasks: {e}");
            }
            Ok(())
        })
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
            commands::templates::push_structured_text,
            commands::images::render_image_preview,
            commands::images::save_image_template,
            commands::images::get_image_thumbnail,
            commands::images::push_image_template,
            commands::images::delete_image_template,
            commands::images::push_sketch,
            commands::page_cache::get_page_cache_list,
            commands::page_cache::delete_page_cache,
            commands::stocks::list_stock_watchlist,
            commands::stocks::add_stock_watch,
            commands::stocks::remove_stock_watch,
            commands::stocks::push_stock_quotes,
            commands::stocks::fetch_stock_quotes,
            commands::stocks::get_stock_push_task,
            commands::stocks::create_stock_push_task,
            commands::stocks::start_stock_push_task,
            commands::stocks::stop_stock_push_task,
            commands::plugins::list_custom_plugins,
            commands::plugins::list_builtin_plugins,
            commands::plugins::save_custom_plugin,
            commands::plugins::delete_custom_plugin,
            commands::plugins::run_plugin_once,
            commands::plugins::push_plugin_once,
            commands::plugins::list_plugin_loop_tasks,
            commands::plugins::create_plugin_loop_task,
            commands::plugins::update_plugin_loop_task,
            commands::plugins::delete_plugin_loop_task,
            commands::plugins::start_plugin_loop_task,
            commands::plugins::stop_plugin_loop_task,
            commands::updater::check_for_update,
            commands::updater::get_current_version,
            commands::image_loop::scan_image_folder,
            commands::image_loop::list_image_loop_tasks,
            commands::image_loop::create_image_loop_task,
            commands::image_loop::update_image_loop_task,
            commands::image_loop::delete_image_loop_task,
            commands::image_loop::start_image_loop_task,
            commands::image_loop::stop_image_loop_task,
            commands::image_loop::push_folder_image,
            commands::image_loop::select_folder_dialog,
            commands::calendar_sync::get_calendar_sync_config,
            commands::calendar_sync::save_calendar_sync_config,
            commands::calendar_sync::list_calendars,
            commands::calendar_sync::sync_calendar,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
