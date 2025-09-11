use crate::modules::types::WindowConfig;
use std::fs;
use tauri::Manager;

/// 保存窗口配置
#[tauri::command]
pub fn save_window_config(config: WindowConfig) -> Result<(), String> {
    let config_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("ton");

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let config_path = config_dir.join("window-config.json");
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, json).map_err(|e| format!("Failed to write config file: {}", e))?;

    Ok(())
}

/// 加载窗口配置
#[tauri::command]
pub fn load_window_config() -> Result<WindowConfig, String> {
    let config_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("ton");

    let config_path = config_dir.join("window-config.json");

    if !config_path.exists() {
        // Return default config if file doesn't exist - 70% screen height, 30% screen width
        return Ok(WindowConfig {
            x: 100.0,
            y: 100.0,
            width: (1920.0 * 0.3),  // 30% of typical screen width
            height: (1080.0 * 0.7), // 70% of typical screen height
        });
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: WindowConfig =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))?;

    Ok(config)
}

/// 显示主窗口
#[tauri::command]
pub fn show_main_window(app: tauri::AppHandle) {
    app.get_webview_window("main").unwrap().show().unwrap();
}

/// 隐藏主窗口
#[tauri::command]
pub fn hide_main_window(app: tauri::AppHandle) {
    app.get_webview_window("main").unwrap().hide().unwrap();
}
