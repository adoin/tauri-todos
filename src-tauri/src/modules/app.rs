use serde_json::Value;

/// 保存应用设置
#[tauri::command]
pub fn save_app_settings(settings: Value) -> Result<(), String> {
    let config_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("Ton")
        .join("config");

    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let config_file = config_dir.join("settings.json");
    let json_str = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    std::fs::write(config_file, json_str)
        .map_err(|e| format!("Failed to write settings file: {}", e))?;

    Ok(())
}

/// 加载应用设置
#[tauri::command]
pub fn load_app_settings() -> Result<Value, String> {
    let config_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("Ton")
        .join("config");

    let config_file = config_dir.join("settings.json");

    if !config_file.exists() {
        // 返回默认设置（不包含宽高）
        return Ok(serde_json::json!({
            "isTransparent": true,
            "showBorder": false,
            "isSettingsOpen": false,
            "activeToolbar": false,
            "locale": "zh-cn",
            "windowConfig": {
                "opacity": 0.8,
                "borderRadius": 8,
                "borderColor": "#3b82f6",
                "borderWidth": 2
            }
        }));
    }

    let json_str = std::fs::read_to_string(config_file)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;

    let settings: Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse settings file: {}", e))?;

    Ok(settings)
}
