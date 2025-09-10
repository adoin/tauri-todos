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
        .join("ton")
        .join("data");

    fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    let todos_file = data_dir.join("todos.json");
    let todos_str = serde_json::to_string_pretty(&todos)
        .map_err(|e| format!("Failed to serialize todos: {}", e))?;

    fs::write(&todos_file, todos_str)
        .map_err(|e| format!("Failed to write todos file: {}", e))?;

    Ok(())
}

// 从文件加载todos
#[tauri::command]
pub fn load_todos() -> Result<Value, String> {
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("ton")
        .join("data");

    let todos_file = data_dir.join("todos.json");
    
    if !todos_file.exists() {
        // 返回默认的空todos数组
        return Ok(serde_json::json!({
            "data": [],
            "lastUpdate": chrono::Utc::now().to_rfc3339()
        }));
    }

    let todos_str = fs::read_to_string(&todos_file)
        .map_err(|e| format!("Failed to read todos file: {}", e))?;

    let todos: Value = serde_json::from_str(&todos_str)
        .map_err(|e| format!("Failed to parse todos file: {}", e))?;

    Ok(todos)
}

// 保存设置到文件
#[tauri::command]
pub fn save_settings(settings: Value) -> Result<(), String> {
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("ton")
        .join("data");

    fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    let settings_file = data_dir.join("settings.json");
    let settings_str = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    fs::write(&settings_file, settings_str)
        .map_err(|e| format!("Failed to write settings file: {}", e))?;

    Ok(())
}

// 从文件加载设置
#[tauri::command]
pub fn load_settings() -> Result<Value, String> {
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("ton")
        .join("data");

    let settings_file = data_dir.join("settings.json");
    
    if !settings_file.exists() {
        // 返回默认设置
        return Ok(serde_json::json!({
            "colors": {
                "primary": "#3b82f6",
                "secondary": "#64748b",
                "accent": "#f59e0b",
                "background": "rgba(255, 255, 255, 0.1)",
                "text": "#ffffff"
            },
            "archive_days": 30,
            "git_sync": {
                "enabled": false,
                "repositoryUrl": "",
                "authMethod": "https",
                "accessToken": "",
                "sshKeyPath": "",
                "autoSync": false
            }
        }));
    }

    let settings_str = fs::read_to_string(&settings_file)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;

    let settings: Value = serde_json::from_str(&settings_str)
        .map_err(|e| format!("Failed to parse settings file: {}", e))?;

    Ok(settings)
}

// 保存应用状态到文件
#[tauri::command]
pub fn save_app_state(app_state: Value) -> Result<(), String> {
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("ton")
        .join("data");

    fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    let app_state_file = data_dir.join("app_state.json");
    let app_state_str = serde_json::to_string_pretty(&app_state)
        .map_err(|e| format!("Failed to serialize app state: {}", e))?;

    fs::write(&app_state_file, app_state_str)
        .map_err(|e| format!("Failed to write app state file: {}", e))?;

    Ok(())
}

// 从文件加载应用状态
#[tauri::command]
pub fn load_app_state() -> Result<Value, String> {
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("ton")
        .join("data");

    let app_state_file = data_dir.join("app_state.json");
    
    if !app_state_file.exists() {
        // 返回默认状态
        return Ok(serde_json::json!({
            "isTransparent": true,
            "windowConfig": {
                "x": 100.0,
                "y": 100.0,
                "width": 400.0,
                "height": 600.0
            },
            "locale": "zh-CN"
        }));
    }

    let app_state_str = fs::read_to_string(&app_state_file)
        .map_err(|e| format!("Failed to read app state file: {}", e))?;

    let app_state: Value = serde_json::from_str(&app_state_str)
        .map_err(|e| format!("Failed to parse app state file: {}", e))?;

    Ok(app_state)
}

// 保存归档的todos到文件
#[tauri::command]
pub fn save_archived_todos(archived_todos: Value) -> Result<(), String> {
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("ton")
        .join("data");

    fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    let archived_file = data_dir.join("archived_todos.json");
    let archived_str = serde_json::to_string_pretty(&archived_todos)
        .map_err(|e| format!("Failed to serialize archived todos: {}", e))?;

    fs::write(&archived_file, archived_str)
        .map_err(|e| format!("Failed to write archived todos file: {}", e))?;

    Ok(())
}

// 从文件加载归档的todos
#[tauri::command]
pub fn load_archived_todos() -> Result<Value, String> {
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("ton")
        .join("data");

    let archived_file = data_dir.join("archived_todos.json");
    
    if !archived_file.exists() {
        // 返回默认的空归档数组
        return Ok(serde_json::json!([]));
    }

    let archived_str = fs::read_to_string(&archived_file)
        .map_err(|e| format!("Failed to read archived todos file: {}", e))?;

    let archived: Value = serde_json::from_str(&archived_str)
        .map_err(|e| format!("Failed to parse archived todos file: {}", e))?;

    Ok(archived)
}
