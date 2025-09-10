use serde::{Deserialize, Serialize};
use std::fs;

// 应用配置相关的数据结构
#[derive(Serialize, Deserialize, Debug)]
pub struct WindowConfig {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppState {
    pub is_transparent: bool,
    pub window_config: WindowConfig,
    pub locale: String,
}

// 保存窗口配置到文件
#[tauri::command]
pub fn save_window_config(config: WindowConfig) -> Result<(), String> {
    let config_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("Ton");

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

// 从文件加载窗口配置
#[tauri::command]
pub fn load_window_config() -> Result<WindowConfig, String> {
    let config_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("Ton");

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

// 获取同步文件列表
#[tauri::command]
pub fn get_sync_files() -> Result<Vec<String>, String> {
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("Ton")
        .join("data");

    let sync_dir = data_dir.join("sync");
    
    if !sync_dir.exists() {
        return Ok(vec![]);
    }

    let mut files = Vec::new();
    
    // 遍历sync目录下的所有文件
    if let Ok(entries) = fs::read_dir(&sync_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    if let Some(file_name) = path.file_name() {
                        if let Some(name_str) = file_name.to_str() {
                            files.push(name_str.to_string());
                        }
                    }
                } else if path.is_dir() {
                    // 如果是目录，遍历其中的文件
                    if let Ok(sub_entries) = fs::read_dir(&path) {
                        for sub_entry in sub_entries {
                            if let Ok(sub_entry) = sub_entry {
                                let sub_path = sub_entry.path();
                                if sub_path.is_file() {
                                    if let Some(file_name) = sub_path.file_name() {
                                        if let Some(name_str) = file_name.to_str() {
                                            if let Some(parent_name) = path.file_name() {
                                                if let Some(parent_str) = parent_name.to_str() {
                                                    files.push(format!("{}/{}", parent_str, name_str));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    files.sort();
    Ok(files)
}
