use serde_json::Value;
use chrono;

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
    
    // 添加 lastUpdate 字段
    let mut settings_with_timestamp = settings;
    if let Some(settings_obj) = settings_with_timestamp.as_object_mut() {
        let current_time = chrono::Utc::now().to_rfc3339();
        settings_obj.insert("lastUpdate".to_string(), serde_json::Value::String(current_time));
    }
    
    let json_str = serde_json::to_string_pretty(&settings_with_timestamp)
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
            "locale": "zh-cn",
            "isTransparent": false,
            "windowConfig": {
              "borderRadius": 8,
              "borderColor": "#3b82f6",
              "borderWidth": 1,
            },
            "colors": {
              "normal": "#4f2937",
              "warning": "#f59e0b",
              "urgent": "#ef4444",
              "completed": "#f5dbd6",
              "background": "#60a5fa",
              "border": "#29cdcd",
            },
            "archiveDays": 30
        }));
    }

    let json_str = std::fs::read_to_string(config_file)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;

    let settings: Value = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse settings file: {}", e))?;

    Ok(settings)
}
