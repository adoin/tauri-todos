// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

// 导入模块
mod modules;

// 重新导出所有命令
use modules::*;
use modules::database::DatabaseState;
use tauri::Manager;

// 简单的问候命令，保留作为示例
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(DatabaseState::default())
        .setup(|app| {
            // 在启动时就设置窗口层级
            let window = app.get_webview_window("main").unwrap();

            #[cfg(target_os = "windows")]
            {
                modules::tray::setup_window_layer(&window);
            }

            // 设置系统托盘
            modules::tray::setup_tray(app)?;

            // 窗口初始为隐藏状态，等待前端配置完成后显示
            // 前端会通过 show_main_window 命令来显示窗口

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            // 窗口管理命令
            save_window_config,
            load_window_config,
            show_main_window,
            hide_main_window,
            // 应用设置命令
            save_app_settings,
            load_app_settings,
            // 数据持久化命令
            save_todos,
            load_todos,
            save_archived_todos,
            load_archived_todos,
            clear_archived_todos,
            // 数据库同步命令
            save_database_config,
            load_database_config,
            test_database_connection,
            connect_database,
            check_and_initialize_tables,
            start_database_sync,
            // 逻辑删除命令
            delete_todo_logically,
            restore_todo,
            restore_todo_safe,
            get_deleted_todos,
            // 数据清理命令
            cleanup_duplicate_data,
            // 数据比较命令
            get_remote_data_for_comparison
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
