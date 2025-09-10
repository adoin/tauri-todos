// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tauri_plugin_dialog::DialogExt;
use serde_json::Value;

// 导入模块
mod modules;

use modules::git;
use modules::todo;
use modules::app;

// 简单的问候命令
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// SSH密钥文件选择命令
#[tauri::command]
fn select_ssh_key_file(app: tauri::AppHandle) -> Result<String, String> {
    // 使用Tauri的文件选择对话框
    let (tx, rx) = std::sync::mpsc::channel();
    
    app.dialog()
        .file()
        .add_filter("SSH Private Key", &["pem", "key", "rsa", "ed25519"])
        .add_filter("All Files", &["*"])
        .set_title("选择SSH私钥文件")
        .pick_file(move |path| {
            let _ = tx.send(path);
        });
    
    match rx.recv() {
        Ok(Some(path)) => {
            let path_str = path.to_string();
            // 验证文件是否存在且可读
            if !std::path::Path::new(&path_str).exists() {
                return Err("选择的文件不存在".to_string());
            }
            Ok(path_str)
        }
        Ok(None) => Err("未选择文件".to_string()),
        Err(_) => Err("文件选择对话框被取消".to_string())
    }
}

// Git同步命令包装
#[tauri::command]
async fn sync_todos_with_git_wrapper(settings: Value) -> Result<String, String> {
    let settings: todo::TodoSettings = serde_json::from_value(settings)
        .map_err(|e| format!("Failed to parse settings: {}", e))?;
    
    git::sync_todos_with_git(settings.git_sync).await
}

pub fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 简化的setup，移除复杂的Windows API调用

            // 创建系统托盘
            let show = MenuItem::with_id(app, "show", "显示", true, None::<&str>)?;
            let hide = MenuItem::with_id(app, "hide", "隐藏", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show, &hide, &settings, &quit])?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .menu(&menu)
                .icon(tauri::image::Image::from_bytes(include_bytes!("../icons/icon.ico")).unwrap())
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "settings" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { 
                        button: tauri::tray::MouseButton::Left, 
                        button_state: tauri::tray::MouseButtonState::Up, 
                        .. 
                    } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            app::save_window_config,
            app::load_window_config,
            todo::save_todos,
            todo::load_todos,
            todo::save_settings,
            todo::load_settings,
            todo::save_app_state,
            todo::load_app_state,
            todo::save_archived_todos,
            todo::load_archived_todos,
            git::initialize_git_sync,
            sync_todos_with_git_wrapper,
            select_ssh_key_file,
            git::check_git_remote_url,
            git::update_git_remote_url,
            git::get_git_sync_status,
            app::get_sync_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}