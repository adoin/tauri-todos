// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn save_app_state(state: Value) -> Result<(), String> {
    let config_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("Ton");

    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let config_file = config_dir.join("app-state.json");
    let json_str = serde_json::to_string_pretty(&state)
        .map_err(|e| format!("Failed to serialize state: {}", e))?;

    std::fs::write(config_file, json_str)
        .map_err(|e| format!("Failed to write state file: {}", e))?;

    Ok(())
}

#[tauri::command]
fn load_app_state() -> Result<Value, String> {
    let config_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("Ton");

    let config_file = config_dir.join("app-state.json");

    if !config_file.exists() {
        // 返回默认状态
        return Ok(serde_json::json!({
            "isTransparent": true,
            "showBorder": false,
            "isSettingsOpen": false,
            "activeToolbar": false,
            "windowConfig": {
                "width": 576,
                "height": 756,
                "opacity": 0.8,
                "borderRadius": 8,
                "borderColor": "#3b82f6",
                "borderWidth": 2
            },
            "windowPosition": {
                "x": 100,
                "y": 100
            }
        }));
    }

    let json_str = std::fs::read_to_string(config_file)
        .map_err(|e| format!("Failed to read state file: {}", e))?;

    let state: Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse state file: {}", e))?;

    Ok(state)
}

use std::fs;
use serde::{Deserialize, Serialize};
use tauri::{
    tray::{TrayIconBuilder, TrayIconEvent},
    menu::{Menu, MenuItem, PredefinedMenuItem},
    Manager, Emitter,
};
use serde_json::Value;

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

// 待办事项相关命令
#[tauri::command]
fn save_todos(todos: Value) -> Result<(), String> {
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("Ton")
        .join("data");

    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    let todo_file = data_dir.join("todos.json");
    let json_str = serde_json::to_string_pretty(&todos)
        .map_err(|e| format!("Failed to serialize todos: {}", e))?;

    std::fs::write(todo_file, json_str)
        .map_err(|e| format!("Failed to write todo file: {}", e))?;

    Ok(())
}

#[tauri::command]
fn save_settings(settings: Value) -> Result<(), String> {
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("Ton")
        .join("data");

    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    let settings_file = data_dir.join("settings.json");
    let json_str = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    std::fs::write(settings_file, json_str)
        .map_err(|e| format!("Failed to write settings file: {}", e))?;

    Ok(())
}

#[tauri::command]
fn load_todos() -> Result<Value, String> {
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("Ton")
        .join("data");

    let todo_file = data_dir.join("todos.json");

    if !todo_file.exists() {
        // 返回默认的空待办事项数组
        return Ok(serde_json::json!([]));
    }

    let json_str = std::fs::read_to_string(todo_file)
        .map_err(|e| format!("Failed to read todo file: {}", e))?;

    let todos: Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse todo file: {}", e))?;

    Ok(todos)
}

#[tauri::command]
fn load_settings() -> Result<Value, String> {
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("Ton")
        .join("data");

    let settings_file = data_dir.join("settings.json");

    if !settings_file.exists() {
        // 返回默认设置
        return Ok(serde_json::json!({
            "colors": {
                "normal": "#1f2937",
                "warning": "#f59e0b",
                "urgent": "#ef4444",
                "completed": "#f5dbd6",
                "background": "#60a5fa88",
                "border": "#e5e7eb",
                "hover": "#f3f4f6"
            },
            "archiveDays": 30
        }));
    }

    let json_str = std::fs::read_to_string(settings_file)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;

    let settings: Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse settings file: {}", e))?;

    Ok(settings)
}

#[tauri::command]
fn save_archived_todos(archived_todos: Value) -> Result<(), String> {
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("Ton")
        .join("data");

    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    let archive_file = data_dir.join("stage.json");
    let json_str = serde_json::to_string_pretty(&archived_todos)
        .map_err(|e| format!("Failed to serialize archived todos: {}", e))?;

    std::fs::write(archive_file, json_str)
        .map_err(|e| format!("Failed to write archive file: {}", e))?;

    Ok(())
}

#[tauri::command]
fn load_archived_todos() -> Result<Value, String> {
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("Ton")
        .join("data");

    let archive_file = data_dir.join("stage.json");

    if !archive_file.exists() {
        return Ok(serde_json::json!({
            "todos": [],
            "archivedAt": ""
        }));
    }

    let json_str = std::fs::read_to_string(archive_file)
        .map_err(|e| format!("Failed to read archive file: {}", e))?;

    let archived_todos: Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse archive file: {}", e))?;

    Ok(archived_todos)
}

#[tauri::command]
fn clear_archived_todos() -> Result<(), String> {
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("Ton")
        .join("data");

    let archive_file = data_dir.join("stage.json");

    if archive_file.exists() {
        std::fs::remove_file(archive_file)
            .map_err(|e| format!("Failed to clear archive file: {}", e))?;
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 在启动时就设置窗口层级
            let window = app.get_webview_window("main").unwrap();
            
            #[cfg(target_os = "windows")]
            {
                use std::ffi::c_void;
                if let Ok(hwnd) = window.hwnd() {
                    unsafe {
                        let hwnd_ptr = hwnd.0 as *mut c_void;
                        let user32 = libloading::Library::new("user32.dll").unwrap();
                        let set_window_pos: libloading::Symbol<unsafe extern "system" fn(*mut c_void, *mut c_void, i32, i32, i32, i32, u32) -> i32> = 
                            user32.get(b"SetWindowPos").unwrap();
                        let _set_window_long: libloading::Symbol<unsafe extern "system" fn(*mut c_void, i32, i32) -> i32> = 
                            user32.get(b"SetWindowLongA").unwrap();
                        let _get_window_long: libloading::Symbol<unsafe extern "system" fn(*mut c_void, i32) -> i32> = 
                            user32.get(b"GetWindowLongA").unwrap();
                        
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
                                                            let _set_window_long: libloading::Symbol<unsafe extern "system" fn(*mut c_void, i32, i32) -> i32> = 
                                                                user32.get(b"SetWindowLongA").unwrap();
                                                            let _get_window_long: libloading::Symbol<unsafe extern "system" fn(*mut c_void, i32) -> i32> = 
                                                                user32.get(b"GetWindowLongA").unwrap();
                                                            
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
                                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, save_window_config, load_window_config, show_main_window, hide_main_window, save_app_state, load_app_state, save_todos, load_todos, save_settings, load_settings, save_archived_todos, load_archived_todos, clear_archived_todos])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
