use serde_json::Value;

/// 保存应用状态
#[tauri::command]
pub fn save_app_state(state: Value) -> Result<(), String> {
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

/// 加载应用状态
#[tauri::command]
pub fn load_app_state() -> Result<Value, String> {
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
