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
            // 在启动时就设置窗口层级
            let window = app.get_webview_window("main").unwrap();

            #[cfg(feature = "windows")]
            {
                use std::ffi::c_void;
                if let Ok(hwnd) = window.hwnd() {
                    unsafe {
                        let hwnd_ptr = hwnd.0 as *mut c_void;
                        let user32 = libloading::Library::new("user32.dll").unwrap();
                        let set_window_pos: libloading::Symbol<
                            unsafe extern "system" fn(
                                *mut c_void,
                                *mut c_void,
                                i32,
                                i32,
                                i32,
                                i32,
                                u32,
                            ) -> i32,
                        > = user32.get(b"SetWindowPos").unwrap();
                        let _set_window_long: libloading::Symbol<
                            unsafe extern "system" fn(*mut c_void, i32, i32) -> i32,
                        > = user32.get(b"SetWindowLongA").unwrap();
                        let _get_window_long: libloading::Symbol<
                            unsafe extern "system" fn(*mut c_void, i32) -> i32,
                        > = user32.get(b"GetWindowLongA").unwrap();

                        // 设置窗口样式，使其不能获得焦点 - 注释掉以允许输入框获得焦点
                        // GWL_EXSTYLE = -20, WS_EX_NOACTIVATE = 0x08000000
                        // let ex_style = get_window_long(hwnd_ptr, -20);
                        // set_window_long(hwnd_ptr, -20, ex_style | 0x08000000);

                        // 设置窗口位置到最底层
                        // HWND_BOTTOM = 1, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE = 0x0013
                        set_window_pos(hwnd_ptr, 1 as *mut c_void, 0, 0, 0, 0, 0x0013);
                    }
                }
            }

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
                            let is_visible = window.is_visible().unwrap_or(false);
                            if is_visible {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                // 不要设置焦点，避免窗口获得焦点而显示在前台
                                // let _ = window.set_focus();

                                // 设置窗口到桌面层级
                                #[cfg(feature = "windows")]
                                {
                                    use std::ffi::c_void;
                                    if let Ok(hwnd) = window.hwnd() {
                                        unsafe {
                                            // 使用 Windows API 设置窗口层级
                                            let hwnd_ptr = hwnd.0 as *mut c_void;
                                            let user32 =
                                                libloading::Library::new("user32.dll").unwrap();
                                            let set_window_pos: libloading::Symbol<
                                                unsafe extern "system" fn(
                                                    *mut c_void,
                                                    *mut c_void,
                                                    i32,
                                                    i32,
                                                    i32,
                                                    i32,
                                                    u32,
                                                )
                                                    -> i32,
                                            > = user32.get(b"SetWindowPos").unwrap();
                                            let _set_window_long: libloading::Symbol<
                                                unsafe extern "system" fn(
                                                    *mut c_void,
                                                    i32,
                                                    i32,
                                                )
                                                    -> i32,
                                            > = user32.get(b"SetWindowLongA").unwrap();
                                            let _get_window_long: libloading::Symbol<
                                                unsafe extern "system" fn(*mut c_void, i32) -> i32,
                                            > = user32.get(b"GetWindowLongA").unwrap();

                                            // 设置窗口样式，使其不能获得焦点 - 注释掉以允许输入框获得焦点
                                            // GWL_EXSTYLE = -20, WS_EX_NOACTIVATE = 0x08000000
                                            // let ex_style = get_window_long(hwnd_ptr, -20);
                                            // set_window_long(hwnd_ptr, -20, ex_style | 0x08000000);

                                            // 设置窗口位置到最底层
                                            // HWND_BOTTOM = 1, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE = 0x0013
                                            set_window_pos(
                                                hwnd_ptr,
                                                1 as *mut c_void,
                                                0,
                                                0,
                                                0,
                                                0,
                                                0x0013,
                                            );
                                        }
                                    }
                                }
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