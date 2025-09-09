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

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};
use tauri_plugin_dialog::DialogExt;
use git2::{Repository, FetchOptions, Signature};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug)]
pub struct WindowConfig {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GitSyncConfig {
    pub enabled: bool,
    #[serde(rename = "repositoryUrl")]
    pub repository_url: String,
    #[serde(rename = "sshKeyPath")]
    pub ssh_key_path: Option<String>,
    #[serde(rename = "autoSync")]
    pub auto_sync: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TodoSettings {
    #[serde(rename = "gitSync")]
    pub git_sync: GitSyncConfig,
}

#[tauri::command]
fn save_window_config(config: WindowConfig) -> Result<(), String> {
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

    std::fs::write(todo_file, json_str).map_err(|e| format!("Failed to write todo file: {}", e))?;

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

    let todos: Value =
        serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse todo file: {}", e))?;

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
            "archiveDays": 30,
            "gitSync": {
                "enabled": false,
                "repositoryUrl": "",
                "autoSync": true
            }
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

// Git同步相关命令
#[tauri::command]
fn initialize_git_sync(repository_url: String, ssh_key_path: Option<String>) -> Result<String, String> {
    println!("Initializing Git sync with URL: {}", repository_url);
    println!("SSH key path: {:?}", ssh_key_path);
    
    // 检查SSH URL是否提供了SSH密钥
    if repository_url.starts_with("git@") || repository_url.starts_with("ssh://") {
        if ssh_key_path.is_none() || ssh_key_path.as_ref().unwrap().is_empty() {
            return Err("SSH URL requires SSH key path. Please set SSH key path first.".to_string());
        }
        
        let ssh_key = ssh_key_path.as_ref().unwrap();
        if !std::path::Path::new(ssh_key).exists() {
            return Err(format!("SSH key file not found: {}", ssh_key));
        }
    }
    
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("Ton")
        .join("data");

    let sync_dir = data_dir.join("sync");
    
    // 如果sync目录已存在，先删除
    if sync_dir.exists() {
        fs::remove_dir_all(&sync_dir)
            .map_err(|e| format!("Failed to remove existing sync directory: {}", e))?;
    }

    // 设置克隆选项
    let mut builder = git2::build::RepoBuilder::new();
    
    // 如果是SSH URL，设置认证回调
    if repository_url.starts_with("git@") || repository_url.starts_with("ssh://") {
        if let Some(ssh_key) = ssh_key_path {
            println!("Setting up SSH authentication with key: {}", ssh_key);
            
            let mut fetch_options = FetchOptions::new();
            let mut callbacks = git2::RemoteCallbacks::new();
            
            // 简化的SSH认证 - 只尝试一次
            let ssh_key_clone = ssh_key.clone();
            callbacks.credentials(move |_url, username_from_url, _allowed_types| {
                println!("SSH authentication attempt with key: {}", ssh_key_clone);
                
                let ssh_key_path = std::path::Path::new(&ssh_key_clone);
                let public_key_path = format!("{}.pub", ssh_key_clone);
                let public_key_exists = std::path::Path::new(&public_key_path).exists();
                
                if public_key_exists {
                    println!("Using SSH key pair: {} + {}", ssh_key_clone, public_key_path);
                    git2::Cred::ssh_key(
                        username_from_url.unwrap_or("git"),
                        Some(std::path::Path::new(&public_key_path)),
                        ssh_key_path,
                        None,
                    )
                } else {
                    println!("Using SSH private key only: {}", ssh_key_clone);
                    git2::Cred::ssh_key(
                        username_from_url.unwrap_or("git"),
                        None,
                        ssh_key_path,
                        None,
                    )
                }
            });
            
            callbacks.certificate_check(|_cert, _valid| {
                Ok(git2::CertificateCheckStatus::CertificateOk)
            });
            
            fetch_options.remote_callbacks(callbacks);
            builder.fetch_options(fetch_options);
        }
    }
    
    // 克隆仓库到sync目录
    let repo = builder.clone(&repository_url, &sync_dir)
        .map_err(|e| format!("Failed to clone repository: {}", e))?;

    // 获取当前分支名称，如果失败则说明是空仓库
    let branch_name = match repo.head() {
        Ok(head) => {
            head.shorthand().unwrap_or("unknown").to_string()
        }
        Err(_) => {
            // 空仓库，直接返回成功，让Git自动处理
            "empty".to_string()
        }
    };

    Ok(format!("Git同步初始化成功，默认分支: {}", branch_name))
}

#[tauri::command]
fn sync_todos_with_git(settings: Value) -> Result<String, String> {
    // 解析设置
    let settings: TodoSettings = serde_json::from_value(settings)
        .map_err(|e| format!("Failed to parse settings: {}", e))?;
    
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("Ton")
        .join("data");

    let sync_dir = data_dir.join("sync");
    
    if !sync_dir.exists() {
        return Err("Sync directory not found. Please initialize Git sync first.".to_string());
    }

    let repo = Repository::open(&sync_dir)
        .map_err(|e| format!("Failed to open repository: {}", e))?;

    // 拉取最新更改
    let mut remote = repo.find_remote("origin")
        .map_err(|e| format!("Failed to find origin remote: {}", e))?;

    let mut fetch_options = FetchOptions::new();
    fetch_options.download_tags(git2::AutotagOption::All);
    
    remote.fetch(&["refs/heads/*:refs/remotes/origin/*"], Some(&mut fetch_options), None)
        .map_err(|e| format!("Failed to fetch from remote: {}", e))?;

    // 检查是否有远程更改需要合并（仅当FETCH_HEAD存在时）
    if let Ok(fetch_head) = repo.find_reference("FETCH_HEAD") {
        if let Ok(fetch_commit) = repo.reference_to_annotated_commit(&fetch_head) {
            let analysis = repo.merge_analysis(&[&fetch_commit])
                .map_err(|e| format!("Failed to analyze merge: {}", e))?;

            if analysis.0.is_up_to_date() {
                // 已经是最新的，继续处理本地数据
            } else if analysis.0.is_fast_forward() {
                // 快进合并 - 使用Git的默认行为
                let head = repo.head()
                    .map_err(|e| format!("Failed to get HEAD: {}", e))?;
                
                let current_branch = head.shorthand()
                    .ok_or("Failed to get current branch name")?;
                
                let mut reference = repo.find_reference(&format!("refs/heads/{}", current_branch))
                    .map_err(|e| format!("Failed to find current branch {}: {}", current_branch, e))?;
                
                reference.set_target(fetch_commit.id(), "Fast-forward")
                    .map_err(|e| format!("Failed to fast-forward: {}", e))?;
                
                repo.set_head(reference.name().unwrap())
                    .map_err(|e| format!("Failed to set HEAD: {}", e))?;
                
                repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                    .map_err(|e| format!("Failed to checkout HEAD: {}", e))?;
            } else {
                // 有冲突，但我们可以继续处理本地数据，稍后推送时再处理
                println!("Warning: Merge conflicts detected, but continuing with local data processing");
            }
        }
    } else {
        // FETCH_HEAD不存在，可能是空仓库，直接继续处理本地数据
        println!("No FETCH_HEAD found, treating as empty repository");
    }

    // 读取本地todos.json的更新日期
    let todos_file = data_dir.join("todos.json");
    let todos_content = fs::read_to_string(&todos_file)
        .map_err(|e| format!("Failed to read todos.json: {}", e))?;
    
    let todos_data: Value = serde_json::from_str(&todos_content)
        .map_err(|e| format!("Failed to parse todos.json: {}", e))?;
    
    let last_update_str = todos_data.get("lastUpdate")
        .and_then(|v| v.as_str())
        .ok_or("Failed to get lastUpdate from todos.json")?;
    
    let last_update = DateTime::parse_from_rfc3339(last_update_str)
        .map_err(|e| format!("Failed to parse lastUpdate: {}", e))?
        .with_timezone(&Utc);
    
    let sync_date = last_update.date_naive();
    
    // 检查是否需要同步
    let sync_path = sync_dir.join(format!("{}/store.json", sync_date.format("%Y-%m-%d")));
    
    if !sync_path.exists() {
        // 创建新的同步文件
        fs::create_dir_all(sync_path.parent().unwrap())
            .map_err(|e| format!("Failed to create sync directory: {}", e))?;
        
        fs::write(&sync_path, &todos_content)
            .map_err(|e| format!("Failed to write sync file: {}", e))?;
    } else {
        // 比较时间戳
        let sync_content = fs::read_to_string(&sync_path)
            .map_err(|e| format!("Failed to read sync file: {}", e))?;
        
        let sync_data: Value = serde_json::from_str(&sync_content)
            .map_err(|e| format!("Failed to parse sync file: {}", e))?;
        
        let sync_last_update_str = sync_data.get("lastUpdate")
            .and_then(|v| v.as_str())
            .ok_or("Failed to get lastUpdate from sync file")?;
        
        let sync_last_update = DateTime::parse_from_rfc3339(sync_last_update_str)
            .map_err(|e| format!("Failed to parse sync lastUpdate: {}", e))?
            .with_timezone(&Utc);
        
        if last_update > sync_last_update {
            // 本地数据更新，写入同步文件
            fs::write(&sync_path, &todos_content)
                .map_err(|e| format!("Failed to update sync file: {}", e))?;
        } else if sync_last_update > last_update {
            // 远程数据更新，更新本地数据
            fs::write(&todos_file, &sync_content)
                .map_err(|e| format!("Failed to update local todos: {}", e))?;
        }
    }

    // 提交并推送更改
    let mut index = repo.index()
        .map_err(|e| format!("Failed to get index: {}", e))?;
    
    index.add_path(Path::new(&format!("{}/store.json", sync_date.format("%Y-%m-%d"))))
        .map_err(|e| format!("Failed to add file to index: {}", e))?;
    
    index.write()
        .map_err(|e| format!("Failed to write index: {}", e))?;
    
    let tree_id = index.write_tree()
        .map_err(|e| format!("Failed to write tree: {}", e))?;
    
    let tree = repo.find_tree(tree_id)
        .map_err(|e| format!("Failed to find tree: {}", e))?;
    
    let signature = Signature::now("Ton App", "ton@example.com")
        .map_err(|e| format!("Failed to create signature: {}", e))?;
    
    // 检查是否是空仓库（没有HEAD或HEAD指向无效的提交）
    let commit_id = match repo.head() {
        Ok(head) => {
            // 有HEAD，尝试找到父提交
            match repo.find_commit(head.target().unwrap()) {
                Ok(parent_commit) => {
                    // 有父提交，正常提交
                    repo.commit(
                        Some("HEAD"),
                        &signature,
                        &signature,
                        &format!("Sync todos for {}", sync_date.format("%Y-%m-%d")),
                        &tree,
                        &[&parent_commit],
                    ).map_err(|e| format!("Failed to commit: {}", e))?
                }
                Err(_) => {
                    // HEAD存在但指向无效提交，创建初始提交
                    repo.commit(
                        Some("HEAD"),
                        &signature,
                        &signature,
                        &format!("Initial sync todos for {}", sync_date.format("%Y-%m-%d")),
                        &tree,
                        &[],
                    ).map_err(|e| format!("Failed to create initial commit: {}", e))?
                }
            }
        }
        Err(_) => {
            // 没有HEAD，创建初始提交
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                &format!("Initial sync todos for {}", sync_date.format("%Y-%m-%d")),
                &tree,
                &[],
            ).map_err(|e| format!("Failed to create initial commit: {}", e))?
        }
    };
    
    // 推送更改 - 使用当前分支
    let mut remote = repo.find_remote("origin")
        .map_err(|e| format!("Failed to find origin remote: {}", e))?;
    
    let mut push_options = git2::PushOptions::new();
    let mut callbacks = git2::RemoteCallbacks::new();
    
    // 设置SSH密钥路径（如果用户指定了）
    if let Some(ssh_key_path) = &settings.git_sync.ssh_key_path {
        if !ssh_key_path.is_empty() && std::path::Path::new(ssh_key_path).exists() {
            let ssh_command = format!("ssh -i \"{}\" -o StrictHostKeyChecking=no", ssh_key_path);
            println!("Setting GIT_SSH_COMMAND: {}", ssh_command);
            std::env::set_var("GIT_SSH_COMMAND", ssh_command);
            
            // 设置SSH认证回调
            let ssh_key_path_clone = ssh_key_path.clone();
            callbacks.credentials(move |url, username_from_url, allowed_types| {
                println!("=== Git2 credentials callback called for sync ===");
                println!("URL: {}", url);
                println!("Username from URL: {:?}", username_from_url);
                println!("Allowed types: {:?}", allowed_types);
                println!("Using SSH key: {}", ssh_key_path_clone);
                
                // 检查SSH密钥文件是否存在
                if !std::path::Path::new(&ssh_key_path_clone).exists() {
                    println!("ERROR: SSH key file does not exist: {}", ssh_key_path_clone);
                    return Err(git2::Error::from_str("SSH key file not found"));
                }
                
                // 尝试SSH密钥认证
                let result = git2::Cred::ssh_key(
                    username_from_url.unwrap_or("git"),
                    None, // 公钥路径（可选）
                    std::path::Path::new(&ssh_key_path_clone),
                    None, // 密码（如果私钥有密码）
                );
                
                match &result {
                    Ok(_) => println!("SSH credentials created successfully"),
                    Err(e) => println!("Failed to create SSH credentials: {}", e),
                }
                
                result
            });
            
            // 添加更多回调来获取详细错误信息
            callbacks.push_update_reference(|refname, status| {
                println!("Push update reference: {} -> {:?}", refname, status);
                Ok(())
            });
            
            callbacks.certificate_check(|cert, valid| {
                println!("Certificate check: valid={}", valid);
                Ok(git2::CertificateCheckStatus::CertificateOk) // 接受所有证书
            });
        } else {
            println!("SSH key path not found or empty: {:?}", ssh_key_path);
        }
    } else {
        println!("No SSH key path specified in settings");
    }
    
    push_options.remote_callbacks(callbacks);
    
    // 获取当前分支名称
    let current_branch = match repo.head() {
        Ok(head) => head.shorthand().unwrap_or("main").to_string(),
        Err(_) => "main".to_string(), // 默认使用main分支
    };
    
    // 推送当前分支到远程同名分支
    let refs = &[format!("refs/heads/{}:refs/heads/{}", current_branch, current_branch)];
    match remote.push(refs, Some(&mut push_options)) {
        Ok(_) => {
            // 推送成功
        }
        Err(e) => {
            // 推送失败，可能是权限问题或者需要先拉取
            println!("Warning: Failed to push changes: {}", e);
            return Ok(format!("Sync completed with local commit: {}. Push failed: {}", commit_id, e));
        }
    }

    Ok(format!("Sync completed successfully. Commit: {}", commit_id))
}

#[tauri::command]
fn select_ssh_key_file(app: tauri::AppHandle) -> Result<String, String> {
    // 使用Tauri的文件选择对话框
    let (tx, rx) = std::sync::mpsc::channel();
    
    app.dialog()
        .file()
        .add_filter("SSH Private Key", &["pem", "key", "rsa", "ed25519"])
        .add_filter("All Files", &["*"])
        .set_title("选择SSH私钥文件")
        .pick_file(move |path| {
            let _ = tx.send(path);
        });
    
    match rx.recv() {
        Ok(Some(path)) => {
            let path_str = path.to_string();
            // 验证文件是否存在且可读
            if !std::path::Path::new(&path_str).exists() {
                return Err("选择的文件不存在".to_string());
            }
            Ok(path_str)
        }
        Ok(None) => Err("未选择文件".to_string()),
        Err(_) => Err("文件选择对话框被取消".to_string())
    }
}

#[tauri::command]
fn check_git_remote_url() -> Result<String, String> {
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("Ton")
        .join("data");

    let sync_dir = data_dir.join("sync");
    
    if !sync_dir.exists() {
        return Err("Sync directory not found. Please initialize Git sync first.".to_string());
    }

    let repo = Repository::open(&sync_dir)
        .map_err(|e| format!("Failed to open repository: {}", e))?;

    let remote = repo.find_remote("origin")
        .map_err(|e| format!("Failed to find origin remote: {}", e))?;
    
    let url = remote.url().unwrap_or("Unknown");
    println!("Current Git remote URL: {}", url);
    
    Ok(url.to_string())
}

#[tauri::command]
fn update_git_remote_url(new_url: String) -> Result<String, String> {
    println!("Updating Git remote URL to: {}", new_url);
    
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("Ton")
        .join("data");

    let sync_dir = data_dir.join("sync");
    
    if !sync_dir.exists() {
        return Err("Sync directory not found. Please initialize Git sync first.".to_string());
    }

    let repo = Repository::open(&sync_dir)
        .map_err(|e| format!("Failed to open repository: {}", e))?;

    // 删除现有的origin远程
    repo.remote_delete("origin")
        .map_err(|e| format!("Failed to delete origin remote: {}", e))?;

    // 添加新的origin远程
    repo.remote("origin", &new_url)
        .map_err(|e| format!("Failed to add origin remote: {}", e))?;

    Ok(format!("Git remote URL updated to: {}", new_url))
}

#[tauri::command]
fn check_local_sync_files() -> Result<Value, String> {
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("Ton")
        .join("data");

    let sync_dir = data_dir.join("sync");
    
    if !sync_dir.exists() {
        return Ok(serde_json::json!({
            "exists": false,
            "message": "Sync directory not found"
        }));
    }

    let mut files = Vec::new();
    
    // 遍历sync目录下的所有文件
    if let Ok(entries) = std::fs::read_dir(&sync_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                    files.push(path.to_string_lossy().to_string());
                } else if path.is_dir() {
                    // 检查子目录中的store.json文件
                    let store_file = path.join("store.json");
                    if store_file.exists() {
                        files.push(store_file.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    Ok(serde_json::json!({
        "exists": true,
        "files": files,
        "count": files.len()
    }))
}


#[tauri::command]
fn test_git_push_auth(settings: Value) -> Result<String, String> {
    // 解析设置
    let settings: TodoSettings = serde_json::from_value(settings)
        .map_err(|e| format!("Failed to parse settings: {}", e))?;
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("Ton")
        .join("data");

    let sync_dir = data_dir.join("sync");
    
    if !sync_dir.exists() {
        return Err("Sync directory not found. Please initialize Git sync first.".to_string());
    }

    let repo = Repository::open(&sync_dir)
        .map_err(|e| format!("Failed to open repository: {}", e))?;

    // 尝试推送来测试推送权限
    let mut remote = repo.find_remote("origin")
        .map_err(|e| format!("Failed to find origin remote: {}", e))?;

    let mut push_options = git2::PushOptions::new();
    let mut callbacks = git2::RemoteCallbacks::new();
    
    // 设置SSH密钥路径（如果用户指定了）
    if let Some(ssh_key_path) = &settings.git_sync.ssh_key_path {
        if !ssh_key_path.is_empty() && std::path::Path::new(ssh_key_path).exists() {
            let ssh_command = format!("ssh -i \"{}\" -o StrictHostKeyChecking=no", ssh_key_path);
            println!("Setting GIT_SSH_COMMAND: {}", ssh_command);
            std::env::set_var("GIT_SSH_COMMAND", ssh_command);
            
            // 设置SSH认证回调
            let ssh_key_path_clone = ssh_key_path.clone();
            callbacks.credentials(move |url, username_from_url, allowed_types| {
                println!("=== Git2 credentials callback called for test ===");
                println!("URL: {}", url);
                println!("Username from URL: {:?}", username_from_url);
                println!("Allowed types: {:?}", allowed_types);
                
                // 检查URL格式
                if url.starts_with("https://") {
                    println!("ERROR: Repository URL is HTTPS format, but SSH key is configured!");
                    println!("HTTPS URLs require Personal Access Token, not SSH keys.");
                    println!("Please change repository URL to SSH format: git@github.com:username/repo.git");
                    return Err(git2::Error::from_str("URL format mismatch: HTTPS URL cannot use SSH key"));
                }
                
                println!("Using SSH key: {}", ssh_key_path_clone);
                
                // 检查SSH密钥文件是否存在
                if !std::path::Path::new(&ssh_key_path_clone).exists() {
                    println!("ERROR: SSH key file does not exist: {}", ssh_key_path_clone);
                    return Err(git2::Error::from_str("SSH key file not found"));
                }
                
                // 尝试SSH密钥认证
                let result = git2::Cred::ssh_key(
                    username_from_url.unwrap_or("git"),
                    None, // 公钥路径（可选）
                    std::path::Path::new(&ssh_key_path_clone),
                    None, // 密码（如果私钥有密码）
                );
                
                match &result {
                    Ok(_) => println!("SSH credentials created successfully"),
                    Err(e) => println!("Failed to create SSH credentials: {}", e),
                }
                
                result
            });
            
            // 添加更多回调来获取详细错误信息
            callbacks.push_update_reference(|refname, status| {
                println!("Push update reference: {} -> {:?}", refname, status);
                Ok(())
            });
            
            callbacks.certificate_check(|cert, valid| {
                println!("Certificate check: valid={}", valid);
                Ok(git2::CertificateCheckStatus::CertificateOk) // 接受所有证书
            });
        } else {
            println!("SSH key path not found or empty: {:?}", ssh_key_path);
        }
    } else {
        println!("No SSH key path specified in settings");
    }
    
    push_options.remote_callbacks(callbacks);
    
    // 获取当前分支名称
    let current_branch = match repo.head() {
        Ok(head) => head.shorthand().unwrap_or("main").to_string(),
        Err(_) => "main".to_string(),
    };
    
    // 尝试推送一个空的引用来测试推送权限
    let refs = &[format!("refs/heads/{}:refs/heads/{}", current_branch, current_branch)];
    
    println!("Attempting to push to remote...");
    match remote.push(refs, Some(&mut push_options)) {
        Ok(_) => {
            println!("Git push successful!");
            Ok("Git push authentication successful!".to_string())
        }
        Err(e) => {
            println!("Git push failed with error: {}", e);
            println!("Error class: {:?}", e.class());
            println!("Error code: {:?}", e.code());
            
            let error_msg = format!("Git push authentication failed: {}", e);
            if error_msg.contains("401") || error_msg.contains("authentication") {
                Err(format!("Push authentication failed (401). Detailed error: {}", e))
            } else if error_msg.contains("403") {
                Err(format!("Push access forbidden (403). Detailed error: {}", e))
            } else if error_msg.contains("non-fast-forward") {
                Err("Push failed: non-fast-forward. This is normal for empty repositories.".to_string())
            } else {
                Err(format!("Push failed: {}", e))
            }
        }
    }
}

#[tauri::command]
fn get_sync_status() -> Result<Value, String> {
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("Ton")
        .join("data");

    let sync_dir = data_dir.join("sync");
    
    if !sync_dir.exists() {
        return Ok(serde_json::json!({
            "initialized": false,
            "message": "Git sync not initialized"
        }));
    }

    let repo = Repository::open(&sync_dir)
        .map_err(|e| format!("Failed to open repository: {}", e))?;

    let head = repo.head()
        .map_err(|e| format!("Failed to get HEAD: {}", e))?;
    
    let commit = repo.find_commit(head.target().unwrap())
        .map_err(|e| format!("Failed to find commit: {}", e))?;
    
    let commit_time = DateTime::from_timestamp(commit.time().seconds(), 0)
        .unwrap_or_default();

    Ok(serde_json::json!({
        "initialized": true,
        "lastCommit": commit_time.to_rfc3339(),
        "branch": head.shorthand().unwrap_or("unknown")
    }))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
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
                        let set_window_pos: libloading::Symbol<
                            unsafe extern "system" fn(
                                *mut c_void,
                                *mut c_void,
                                i32,
                                i32,
                                i32,
                                i32,
                                u32,
                            ) -> i32,
                        > = user32.get(b"SetWindowPos").unwrap();
                        let _set_window_long: libloading::Symbol<
                            unsafe extern "system" fn(*mut c_void, i32, i32) -> i32,
                        > = user32.get(b"SetWindowLongA").unwrap();
                        let _get_window_long: libloading::Symbol<
                            unsafe extern "system" fn(*mut c_void, i32) -> i32,
                        > = user32.get(b"GetWindowLongA").unwrap();

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

            let menu = Menu::with_items(
                app,
                &[
                    &show,
                    &hide,
                    &PredefinedMenuItem::separator(app)?,
                    &settings,
                    &PredefinedMenuItem::separator(app)?,
                    &quit,
                ],
            )?;

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
                    if let TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        ..
                    } = event
                    {
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
                                            let user32 =
                                                libloading::Library::new("user32.dll").unwrap();
                                            let set_window_pos: libloading::Symbol<
                                                unsafe extern "system" fn(
                                                    *mut c_void,
                                                    *mut c_void,
                                                    i32,
                                                    i32,
                                                    i32,
                                                    i32,
                                                    u32,
                                                )
                                                    -> i32,
                                            > = user32.get(b"SetWindowPos").unwrap();
                                            let _set_window_long: libloading::Symbol<
                                                unsafe extern "system" fn(
                                                    *mut c_void,
                                                    i32,
                                                    i32,
                                                )
                                                    -> i32,
                                            > = user32.get(b"SetWindowLongA").unwrap();
                                            let _get_window_long: libloading::Symbol<
                                                unsafe extern "system" fn(*mut c_void, i32) -> i32,
                                            > = user32.get(b"GetWindowLongA").unwrap();

                                            // 设置窗口样式，使其不能获得焦点 - 注释掉以允许输入框获得焦点
                                            // GWL_EXSTYLE = -20, WS_EX_NOACTIVATE = 0x08000000
                                            // let ex_style = get_window_long(hwnd_ptr, -20);
                                            // set_window_long(hwnd_ptr, -20, ex_style | 0x08000000);

                                            // 设置窗口位置到最底层
                                            // HWND_BOTTOM = 1, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE = 0x0013
                                            set_window_pos(
                                                hwnd_ptr,
                                                1 as *mut c_void,
                                                0,
                                                0,
                                                0,
                                                0,
                                                0x0013,
                                            );
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
        .invoke_handler(tauri::generate_handler![
            greet,
            save_window_config,
            load_window_config,
            show_main_window,
            hide_main_window,
            save_app_state,
            load_app_state,
            save_todos,
            load_todos,
            save_settings,
            load_settings,
            save_archived_todos,
            load_archived_todos,
            clear_archived_todos,
            initialize_git_sync,
            sync_todos_with_git,
            get_sync_status,
            test_git_push_auth,
            check_local_sync_files,
            select_ssh_key_file,
            check_git_remote_url,
            update_git_remote_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
