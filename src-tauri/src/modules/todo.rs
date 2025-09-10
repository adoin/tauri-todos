use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;

// Todo相关的数据结构
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TodoItem {
    pub id: String,
    pub text: String,
    pub completed: bool,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub priority: i32,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TodoSettings {
    pub colors: TodoColors,
    pub archive_days: i32,
    pub git_sync: GitSyncConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TodoColors {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub background: String,
    pub text: String,
}

// 使用git模块中的GitSyncConfig
pub use crate::modules::git::GitSyncConfig;

// 保存todos到文件
#[tauri::command]
pub fn save_todos(todos: Value) -> Result<(), String> {
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("Ton")
        .join("data");

    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    let todo_file = data_dir.join("todos.json");
    let json_str = serde_json::to_string_pretty(&todos)
        .map_err(|e| format!("Failed to serialize todos: {}", e))?;

    std::fs::write(todo_file, json_str).map_err(|e| format!("Failed to write todo file: {}", e))?;

    Ok(())
}

// 从文件加载todos
#[tauri::command]
pub fn load_todos() -> Result<Value, String> {
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

    let todos: Value =
        serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse todo file: {}", e))?;

    Ok(todos)
}

// 保存设置到文件
#[tauri::command]
pub fn save_settings(settings: Value) -> Result<(), String> {
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

// 从文件加载设置
#[tauri::command]
pub fn load_settings() -> Result<Value, String> {
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

// 保存应用状态到文件
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

// 从文件加载应用状态
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

// 保存归档的todos到文件
#[tauri::command]
pub fn save_archived_todos(archived_todos: Value) -> Result<(), String> {
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

// 从文件加载归档的todos
#[tauri::command]
pub fn load_archived_todos() -> Result<Value, String> {
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
