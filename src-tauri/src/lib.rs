// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

// 导入模块
mod modules;

// 重新导出所有命令
use modules::*;
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
        .setup(|app| {
            // 在启动时就设置窗口层级
            let window = app.get_webview_window("main").unwrap();

            #[cfg(target_os = "windows")]
            {
                modules::tray::setup_window_layer(&window);
            }

            // 设置系统托盘
            modules::tray::setup_tray(app)?;

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
            clear_archived_todos
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
