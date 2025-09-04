// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

use std::fs;
use serde::{Deserialize, Serialize};
use tauri::{
    tray::{TrayIconBuilder, TrayIconEvent},
    menu::{Menu, MenuItem, PredefinedMenuItem},
    Manager, Emitter,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct WindowConfig {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[tauri::command]
fn save_window_config(config: WindowConfig) -> Result<(), String> {
    let config_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("ton");

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let config_path = config_dir.join("window-config.json");
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, json)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

#[tauri::command]
fn load_window_config() -> Result<WindowConfig, String> {
    let config_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("ton");

    let config_path = config_dir.join("window-config.json");

    if !config_path.exists() {
        // Return default config if file doesn't exist - 70% screen height, 30% screen width
        return Ok(WindowConfig {
            x: 100.0,
            y: 100.0,
            width: (1920.0 * 0.3), // 30% of typical screen width
            height: (1080.0 * 0.7), // 70% of typical screen height
        });
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: WindowConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    Ok(config)
}

#[tauri::command]
fn show_main_window(app: tauri::AppHandle) {
    app.get_webview_window("main").unwrap().show().unwrap();
}

#[tauri::command]
fn hide_main_window(app: tauri::AppHandle) {
    app.get_webview_window("main").unwrap().hide().unwrap();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
            let show = MenuItem::with_id(app, "show", "显示", true, None::<&str>)?;
            let hide = MenuItem::with_id(app, "hide", "隐藏", true, None::<&str>)?;
            
            let menu = Menu::with_items(app, &[
                &show,
                &hide,
                &PredefinedMenuItem::separator(app)?,
                &settings,
                &PredefinedMenuItem::separator(app)?,
                &quit,
            ])?;

            // 直接使用原始字节数据创建图标
            let icon_bytes = include_bytes!("../icons/icon.ico");
            let icon = tauri::image::Image::from_bytes(icon_bytes)?;
            
            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(icon)
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "settings" => {
                        let window = app.get_webview_window("main").unwrap();
                        window.show().unwrap();
                        window.set_focus().unwrap();
                        window.emit("open-settings", {}).unwrap();
                    }
                    "show" => {
                        let window = app.get_webview_window("main").unwrap();
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                    "hide" => {
                        let window = app.get_webview_window("main").unwrap();
                        window.hide().unwrap();
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
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
                                                #[cfg(target_os = "windows")]
                                                {
                                                    use std::ffi::c_void;
                                                    if let Ok(hwnd) = window.hwnd() {
                                                        unsafe {
                                                            // 使用 Windows API 设置窗口层级
                                                            let hwnd_ptr = hwnd.0 as *mut c_void;
                                                            let user32 = libloading::Library::new("user32.dll").unwrap();
                                                            let set_window_pos: libloading::Symbol<unsafe extern "system" fn(*mut c_void, *mut c_void, i32, i32, i32, i32, u32) -> i32> = 
                                                                user32.get(b"SetWindowPos").unwrap();
                                                            let set_window_long: libloading::Symbol<unsafe extern "system" fn(*mut c_void, i32, i32) -> i32> = 
                                                                user32.get(b"SetWindowLongA").unwrap();
                                                            let get_window_long: libloading::Symbol<unsafe extern "system" fn(*mut c_void, i32) -> i32> = 
                                                                user32.get(b"GetWindowLongA").unwrap();
                                                            
                                                            // 设置窗口样式，使其不能获得焦点
                                                            // GWL_EXSTYLE = -20, WS_EX_NOACTIVATE = 0x08000000
                                                            let ex_style = get_window_long(hwnd_ptr, -20);
                                                            set_window_long(hwnd_ptr, -20, ex_style | 0x08000000);
                                                            
                                                            // 设置窗口位置到最底层
                                                            // HWND_BOTTOM = 1, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE = 0x0013
                                                            set_window_pos(hwnd_ptr, 1 as *mut c_void, 0, 0, 0, 0, 0x0013);
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
        .invoke_handler(tauri::generate_handler![greet, save_window_config, load_window_config, show_main_window, hide_main_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
