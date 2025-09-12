use serde_json::Value;

/// 保存待办事项数据
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

/// 加载待办事项数据
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

/// 保存已归档的待办事项
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

/// 加载已归档的待办事项
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

/// 清空已归档的待办事项
#[tauri::command]
pub fn clear_archived_todos() -> Result<(), String> {
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
