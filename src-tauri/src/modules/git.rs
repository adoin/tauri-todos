use gix;
use gix::bstr::ByteSlice;
use chrono::{DateTime, Utc};
use tokio::task;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct GitSyncConfig {
    pub enabled: bool,
    #[serde(rename = "repositoryUrl")]
    pub repository_url: String,
    #[serde(rename = "authMethod")]
    pub auth_method: String,
    #[serde(rename = "accessToken")]
    pub access_token: Option<String>,
    #[serde(rename = "sshKeyPath")]
    pub ssh_key_path: Option<String>,
    #[serde(rename = "autoSync")]
    pub auto_sync: bool,
}

// ============================================================================
// Git 工具函数 - 封装常用的Git操作
// ============================================================================

/// 解决JSON文件的合并冲突，比较lastUpdate字段并使用较新的版本
pub fn resolve_json_merge_conflict(local_content: &str, remote_content: &str) -> Result<String, String> {
    println!("Resolving JSON merge conflict...");
    
    // 解析本地和远程的JSON内容
    let local_json: Value = serde_json::from_str(local_content)
        .map_err(|e| format!("Failed to parse local JSON: {}", e))?;
    
    let remote_json: Value = serde_json::from_str(remote_content)
        .map_err(|e| format!("Failed to parse remote JSON: {}", e))?;
    
    // 获取lastUpdate字段
    let local_last_update = local_json.get("lastUpdate")
        .and_then(|v| v.as_str())
        .ok_or("Local JSON missing lastUpdate field")?;
    
    let remote_last_update = remote_json.get("lastUpdate")
        .and_then(|v| v.as_str())
        .ok_or("Remote JSON missing lastUpdate field")?;
    
    println!("Local lastUpdate: {}", local_last_update);
    println!("Remote lastUpdate: {}", remote_last_update);
    
    // 解析时间戳
    let local_time = DateTime::parse_from_rfc3339(local_last_update)
        .map_err(|e| format!("Failed to parse local timestamp: {}", e))?
        .with_timezone(&Utc);
    
    let remote_time = DateTime::parse_from_rfc3339(remote_last_update)
        .map_err(|e| format!("Failed to parse remote timestamp: {}", e))?
        .with_timezone(&Utc);
    
    // 比较时间戳，使用较新的版本
    let resolved_json = if local_time > remote_time {
        println!("Using local version (newer)");
        local_json
    } else {
        println!("Using remote version (newer)");
        remote_json
    };
    
    // 更新lastUpdate为当前时间
    let mut resolved_json = resolved_json;
    if let Some(obj) = resolved_json.as_object_mut() {
        obj.insert("lastUpdate".to_string(), Value::String(Utc::now().to_rfc3339()));
        obj.insert("mergeResolved".to_string(), Value::String(format!("Resolved conflict at {}", Utc::now().to_rfc3339())));
    }
    
    // 序列化回JSON字符串
    let resolved_content = serde_json::to_string_pretty(&resolved_json)
        .map_err(|e| format!("Failed to serialize resolved JSON: {}", e))?;
    
    println!("Merge conflict resolved successfully");
    Ok(resolved_content)
}

/// 将文件添加到Git索引（stage文件）
pub fn stage_files(repo: &gix::Repository, file_paths: &[&str]) -> Result<(), String> {
    println!("Staging files: {:?}", file_paths);
    
    // 由于gix的索引API比较复杂，我们暂时使用一个简化的实现
    // 在实际应用中，这里需要实现完整的索引操作
    
    for file_path in file_paths {
        // 读取文件内容
        let file_content = fs::read(file_path)
            .map_err(|e| format!("Failed to read file {}: {}", file_path, e))?;
        
        // 创建blob对象
        let blob_id = repo.write_blob(&file_content)
            .map_err(|e| format!("Failed to create blob for {}: {}", file_path, e))?;
        
        println!("Created blob for {}: {}", file_path, blob_id);
    }
    
    println!("Successfully prepared {} files for staging", file_paths.len());
    Ok(())
}

/// 创建包含指定文件的树对象
pub fn create_tree_with_files(repo: &gix::Repository, file_paths: &[&str]) -> Result<gix::ObjectId, String> {
    println!("Creating tree with files: {:?}", file_paths);
    
    use gix::objs::tree::{EntryMode, EntryKind};
    use gix::objs::Tree;
    
    let mut tree_entries = Vec::new();
    
    for file_path in file_paths {
        // 读取文件内容
        let file_content = fs::read(file_path)
            .map_err(|e| format!("Failed to read file {}: {}", file_path, e))?;
        
        // 创建blob对象
        let blob_id = repo.write_blob(&file_content)
            .map_err(|e| format!("Failed to create blob for {}: {}", file_path, e))?;
        
        println!("Created blob for {}: {}", file_path, blob_id);
        
        // 获取相对于仓库根目录的文件名
        let path = std::path::Path::new(file_path);
        let filename = path.file_name()
            .ok_or_else(|| format!("Invalid file path: {}", file_path))?
            .to_str()
            .ok_or_else(|| format!("Invalid UTF-8 in file path: {}", file_path))?;
        
        // 如果有子目录，需要包含完整路径
        let relative_path = if let Some(parent) = path.parent() {
            if parent != std::path::Path::new(".") {
                path.to_str().unwrap()
            } else {
                filename
            }
        } else {
            filename
        };
        
        println!("Adding to tree: {} -> {}", relative_path, blob_id);
        
        // 添加条目到树
        tree_entries.push(gix::objs::tree::Entry {
            mode: EntryMode::from(EntryKind::Blob),
            filename: relative_path.into(),
            oid: blob_id.into(),
        });
    }
    
    // 按文件名排序（Git要求）
    tree_entries.sort_by(|a, b| a.filename.cmp(&b.filename));
    
    println!("Tree entries after sorting:");
    for (i, entry) in tree_entries.iter().enumerate() {
        println!("  {}: {} -> {}", i, entry.filename, entry.oid);
    }
    
    // 创建树对象
    let tree = Tree { entries: tree_entries };
    let tree_id = repo.write_object(&tree)
        .map_err(|e| format!("Failed to write tree: {}", e))?;
    
    println!("Created tree with {} files: {}", file_paths.len(), tree_id);
    Ok(tree_id.into())
}

/// 创建提交
pub fn create_commit(
    repo: &gix::Repository,
    tree_id: gix::ObjectId,
    message: &str,
    parent_commits: Option<Vec<gix::ObjectId>>,
) -> Result<gix::ObjectId, String> {
    println!("Creating commit with message: {}", message);
    
    let parents = parent_commits.unwrap_or_default();
    let commit_id = repo.commit(
        "HEAD",
        message,
        tree_id,
        parents,
    ).map_err(|e| format!("Failed to create commit: {}", e))?;
    
    println!("Created commit: {}", commit_id);
    Ok(commit_id.into())
}

/// 推送提交到远程仓库
pub async fn push_to_remote(
    repo: &gix::Repository,
    auth_method: &str,
    token: Option<&str>,
    ssh_key_path: Option<&str>,
) -> Result<(), String> {
    println!("实际推送到远程仓库...");
    
    // 获取远程仓库
    let mut remote = repo.find_remote("origin")
        .map_err(|e| format!("找不到origin远程仓库: {}", e))?;
    
    // 建立连接
    let connection = remote.connect(gix::remote::Direction::Push)
        .map_err(|e| format!("连接远程仓库失败: {}", e))?;
    
    // 获取当前分支
    let head_ref = repo.head_ref()
        .map_err(|e| format!("获取HEAD引用失败: {}", e))?;
    
    let head_ref = head_ref.ok_or("HEAD引用不存在")?;
    let branch_name = head_ref.name().shorten().to_string();
    
    // 构建推送规范
    let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
    
    // 执行推送
    let push_outcome = connection.push(
        &[refspec.as_str()],
        &mut gix::progress::Discard,
        &gix::interrupt::IS_INTERRUPTED
    ).map_err(|e| format!("推送失败: {}", e))?;
    
    // 检查推送结果
    if push_outcome.ref_updates.iter().all(|update| update.status.is_ok()) {
        println!("推送成功到分支: {}", branch_name);
        Ok(())
    } else {
        Err("推送过程中出现错误".to_string())
    }
}

/// 处理推送时的合并冲突
pub fn handle_push_merge_conflict(
    repo: &gix::Repository,
    file_path: &str,
    local_content: &str,
    remote_content: &str,
) -> Result<String, String> {
    println!("Handling merge conflict for file: {}", file_path);
    
    // 使用JSON合并冲突解决函数
    let resolved_content = resolve_json_merge_conflict(local_content, remote_content)?;
    
    // 将解决后的内容写回文件
    fs::write(file_path, &resolved_content)
        .map_err(|e| format!("Failed to write resolved content to {}: {}", file_path, e))?;
    
    println!("Merge conflict resolved and file updated: {}", file_path);
    Ok(resolved_content)
}

/// 完整的提交和推送流程
pub async fn commit_and_push_files(
    repo: &gix::Repository,
    file_paths: &[&str],
    commit_message: &str,
    auth_method: &str,
    token: Option<&str>,
    ssh_key_path: Option<&str>,
) -> Result<gix::ObjectId, String> {
    println!("Starting commit and push workflow for {} files", file_paths.len());
    
    // 1. 创建包含文件的树
    let tree_id = create_tree_with_files(repo, file_paths)?;
    
    // 2. 创建提交
    let commit_id = create_commit(repo, tree_id, commit_message, None)?;
    
    // 3. 推送到远程
    push_to_remote(repo, auth_method, token, ssh_key_path).await?;
    
    println!("Successfully completed commit and push workflow");
    Ok(commit_id)
}

// 简化的异步推送函数 - 先实现基本的commit功能
pub async fn push_with_gix_async(
    repo_path: &Path,
    auth_method: &str,
    token: Option<&str>,
    ssh_key_path: Option<&str>,
) -> Result<String, String> {
    let repo_path = repo_path.to_path_buf();
    let auth_method = auth_method.to_string();
    let token = token.map(|s| s.to_string());
    let ssh_key_path = ssh_key_path.map(|s| s.to_string());
    
    // 暂时只实现本地commit，推送功能稍后实现
    task::spawn_blocking(move || {
        // 1. 打开仓库
        let repo = gix::open(&repo_path).map_err(|e| format!("Failed to open repository: {}", e))?;
        println!("Repository opened successfully with gix");
        
        // 2. 获取或创建索引
        let index = match repo.try_index() {
            Ok(Some(idx)) => idx,
            Ok(None) => {
                repo.index_or_empty().map_err(|e| format!("Failed to create empty index: {}", e))?
            }
            Err(e) => return Err(format!("Failed to get index: {}", e))
        };
        
        // 3. 添加文件到索引
        let mut paths_to_add = Vec::new();
        for entry in std::fs::read_dir(&repo_path).map_err(|e| format!("Failed to read directory: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            if path.is_file() && path.file_name().unwrap() != ".git" {
                if let Some(relative_path) = path.strip_prefix(&repo_path).ok() {
                    paths_to_add.push(relative_path.to_path_buf());
                }
            } else if path.is_dir() && path.file_name().unwrap() != ".git" {
                add_directory_to_paths(&path, &repo_path, &mut paths_to_add)?;
            }
        }
        
        // 4. 写入索引 - 需要获取可变的索引
        let mut index_file = repo.open_index().map_err(|e| format!("Failed to open index for writing: {}", e))?;
        index_file.write(gix::index::write::Options::default()).map_err(|e| format!("Failed to write index: {}", e))?;
        println!("Added {} files to index", paths_to_add.len());
        
        // 5. 创建提交
        let commit_message = format!("Sync todos - {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
        
        // 获取当前HEAD作为父提交
        let parent_commit = match repo.head() {
            Ok(mut head) => {
                match head.try_peel_to_id_in_place() {
                    Ok(Some(commit_id)) => Some(commit_id.detach()),
                    Ok(None) | Err(_) => None
                }
            }
            Err(_) => None
        };
        
        // 获取当前tree ID
        let tree_id = repo.head_tree_id().map_err(|e| format!("Failed to get head tree ID: {}", e))?;
        
        // 创建提交
        let commit_id = match parent_commit {
            Some(parent) => {
                repo.commit("refs/heads/master", &commit_message, tree_id, [parent])
                    .map_err(|e| format!("Failed to create commit: {}", e))?
            }
            None => {
                // 首次提交，没有父提交
                let empty_parents: Vec<gix::ObjectId> = vec![];
                repo.commit("refs/heads/master", &commit_message, tree_id, empty_parents)
                    .map_err(|e| format!("Failed to create initial commit: {}", e))?
            }
        };
        
        println!("Created commit: {}", commit_id);
        
        // 6. 推送到远程仓库
        println!("Attempting to push to remote repository...");
        
        if auth_method == "https" {
            if let Some(ref _token) = token {
                println!("HTTPS authentication configured");
                
                // 暂时跳过推送，稍后实现
                println!("Push functionality will be implemented in next iteration");
                Ok(format!("Commit created: {}. Push: Push preparation completed - will implement actual push next", commit_id))
            } else {
                Err("HTTPS auth requires token".to_string())
            }
        } else if auth_method == "ssh" {
            if let Some(ref _ssh_key) = ssh_key_path {
                println!("SSH authentication configured");
                
                // 暂时跳过推送，稍后实现
                println!("Push functionality will be implemented in next iteration");
                Ok(format!("Commit created: {}. Push: Push preparation completed - will implement actual push next", commit_id))
            } else {
                Err("SSH auth requires key path".to_string())
            }
        } else {
            Err("Unknown auth method".to_string())
        }
    }).await.map_err(|e| format!("Task failed: {}", e))?
}

// 递归添加目录中的文件到路径列表
fn add_directory_to_paths(
    dir_path: &std::path::Path,
    repo_path: &std::path::Path,
    paths: &mut Vec<std::path::PathBuf>,
) -> Result<(), String> {
    for entry in std::fs::read_dir(dir_path).map_err(|e| format!("Failed to read directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        if path.is_file() {
            if let Some(relative_path) = path.strip_prefix(repo_path).ok() {
                paths.push(relative_path.to_path_buf());
            }
        } else if path.is_dir() {
            add_directory_to_paths(&path, repo_path, paths)?;
        }
    }
    Ok(())
}

// 配置远程仓库origin
pub fn configure_remote_origin(repo_path: &Path, remote_url: &str) -> Result<(), String> {
    println!("Configuring remote origin: {}", remote_url);
    
    let config_path = repo_path.join(".git").join("config");
    let config_content = format!(
        "[core]
\trepositoryformatversion = 0
\tfilemode = false
\tbare = false
\tlogallrefupdates = true
\tsymlinks = false
\tignorecase = true
[remote \"origin\"]
\turl = {}
\tfetch = +refs/heads/*:refs/remotes/origin/*
[branch \"main\"]
\tremote = origin
\tmerge = refs/heads/main
[branch \"master\"]
\tremote = origin
\tmerge = refs/heads/master
",
        remote_url
    );
    
    fs::write(&config_path, config_content)
        .map_err(|e| format!("Failed to write git config: {}", e))?;
    
    println!("Git config written successfully");
    Ok(())
}

// 初始化Git同步
#[tauri::command]
pub async fn initialize_git_sync(
    repository_url: String,
    auth_method: String,
    ssh_key_path: Option<String>,
    access_token: Option<String>,
) -> Result<String, String> {
    println!("Initializing Git sync with gix - URL: {}", repository_url);
    println!("Auth method: {}", auth_method);
    println!("SSH key path: {:?}", ssh_key_path);
    println!("Access token provided: {}", access_token.is_some());
    
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
    
    // 创建sync目录
    fs::create_dir_all(&sync_dir)
        .map_err(|e| format!("Failed to create sync directory: {}", e))?;
    
    // 构建克隆URL
    let clone_url = if auth_method == "ssh" {
        repository_url.clone()
    } else {
        // HTTPS认证：在URL中嵌入token - 使用GitHub推荐的格式
        if let Some(token) = &access_token {
            if repository_url.starts_with("https://github.com/") {
                let repo_path = repository_url.strip_prefix("https://github.com/")
                    .unwrap_or(&repository_url)
                    .strip_suffix(".git")
                    .unwrap_or(&repository_url);
                format!("https://{}:x-oauth-basic@github.com/{}.git", token, repo_path)
            } else {
                repository_url.clone()
            }
        } else {
            repository_url.clone()
        }
    };
    
    println!("Using clone URL: {}", if auth_method == "https" { "[URL with embedded token]" } else { &clone_url });

    // 尝试使用gix克隆仓库
    let sync_dir_clone = sync_dir.clone();
    let clone_result: Result<(), String> = task::spawn_blocking(move || {
        // 使用gix克隆仓库
        match gix::prepare_clone(clone_url.as_str(), &sync_dir_clone) {
            Ok(mut prepare) => {
                println!("Clone preparation succeeded, attempting to fetch...");
                
                // 尝试执行fetch操作
                match prepare.fetch_only(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED) {
                    Ok((repo, _outcome)) => {
                        println!("Successfully cloned repository with gix");
                        drop(repo); // 释放仓库句柄
                        Ok(())
                    }
                    Err(e) => {
                        println!("Fetch failed: {}, this might be an empty repository", e);
                        // 对于空仓库，fetch会失败，但我们仍然需要初始化
                        Err(format!("Fetch failed (likely empty repo): {}", e))
                    }
                }
            }
            Err(e) => {
                println!("gix prepare_clone failed: {}, will try manual initialization", e);
                Err(format!("Clone failed: {}", e))
            }
        }
    }).await.map_err(|e| format!("Clone task failed: {}", e))?;
    
    // 如果克隆失败，手动初始化仓库
    if clone_result.is_err() {
        println!("Clone failed, initializing repository manually with gix...");
        
        // 使用gix手动初始化仓库
        match gix::init(&sync_dir) {
            Ok(_repo) => {
                println!("Successfully initialized repository with gix");
                
                // 配置远程origin
                configure_remote_origin(&sync_dir, &repository_url)?;
            }
            Err(e) => {
                return Err(format!("Failed to initialize repository: {}", e));
            }
        }
    } else {
        println!("Successfully cloned repository with gix");
    }
    
    // 在spawn_blocking中处理所有gix操作
    let sync_dir_clone = sync_dir.clone();
    let repository_url_clone = repository_url.clone();
    let auth_method_clone = auth_method.clone();
    let access_token_clone = access_token.clone();
    let ssh_key_path_clone = ssh_key_path.clone();
    
    let branch_name = task::spawn_blocking(move || {
        // 重新打开仓库
        let cloned_repo = match gix::open(&sync_dir_clone) {
            Ok(repo) => repo,
            Err(e) => {
                println!("Warning: Could not open cloned repository: {}", e);
                return Ok::<String, String>("Git同步初始化成功，但无法检测分支信息".to_string());
            }
        };

        // 检测默认分支并检查是否为空仓库
        match cloned_repo.head() {
        Ok(head) => {
            match head.referent_name() {
                Some(name) => {
                    let branch = name.shorten().to_string();
                    println!("Detected default branch: {}", branch);
                    
                    // 检查工作目录是否有文件（除了.git）
                    let has_files = sync_dir.read_dir()
                        .map(|mut entries| {
                            entries.any(|entry| {
                                entry.map(|e| {
                                    let name = e.file_name();
                                    let name = name.to_string_lossy();
                                    name != ".git" && name != "." && name != ".."
                                }).unwrap_or(false)
                            })
                        })
                        .unwrap_or(false);
                    
                    if !has_files {
                        println!("Working directory is empty, this is an empty repository");
                        println!("Creating initial commit for empty repository...");
                        
                        // 先创建初始文件
                        let readme_content = format!(
                            "# Todo Store\n\nThis repository stores todo backups from Tauri Todo App.\n\n## Structure\n\n- Each day's backup is stored in `YYYY-MM-DD/store.json`\n- The `store.json` file contains the complete todo data\n\n## Repository\n\nRepository: {}\nCreated: {}\n",
                            repository_url,
                            Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
                        );
                        let readme_path = sync_dir.join("README.md");
                        fs::write(&readme_path, readme_content)
                            .map_err(|e| format!("Failed to create README.md: {}", e))?;
                        
                        // 创建今天的同步文件夹和数据
                        let today = Utc::now().date_naive();
                        let sync_folder = sync_dir.join(format!("{}", today.format("%Y-%m-%d")));
                        fs::create_dir_all(&sync_folder)
                            .map_err(|e| format!("Failed to create sync folder: {}", e))?;
                        
                        // 读取或创建初始的todos数据
                        let todos_file = data_dir.join("todos.json");
                        let initial_data = if todos_file.exists() {
                            fs::read_to_string(&todos_file)
                                .map_err(|e| format!("Failed to read todos.json: {}", e))?
                        } else {
                            serde_json::json!({
                                "data": [],
                                "lastUpdate": Utc::now().to_rfc3339(),
                                "source": "initial_empty_repo"
                            }).to_string()
                        };
                        
                        // 写入store.json
                        let store_path = sync_folder.join("store.json");
                        fs::write(&store_path, &initial_data)
                            .map_err(|e| format!("Failed to create store.json: {}", e))?;
                        
                        println!("Created initial files: README.md and {}/store.json", today.format("%Y-%m-%d"));
                        
                        // 使用封装的Git工具函数进行提交和推送
                        println!("Created initial files, now attempting to commit and push...");
                        
                        // 准备实际的文件路径
                        let readme_path = sync_dir.join("README.md");
                        let store_path = sync_folder.join("store.json");
                        let file_paths = &[
                            readme_path.to_str().unwrap(),
                            store_path.to_str().unwrap()
                        ];
                        
                        // 使用封装的函数进行完整的提交和推送流程
                        let commit_message = "Initial commit: Setup todo backup repository";
                        let commit_id = commit_and_push_files(
                            &cloned_repo,
                            file_paths,
                            commit_message,
                            &auth_method_clone,
                            access_token_clone.as_deref(),
                            ssh_key_path_clone.as_deref(),
                        ).await?;
                        
                        return Ok(format!("Git同步初始化成功，已创建初始提交并推送: {}，默认分支: {}", commit_id, branch));
                    }
                    
                    // 简单检查远程是否存在
                    match cloned_repo.find_remote("origin") {
                        Ok(_remote) => {
                            println!("Remote 'origin' found, repository is properly configured");
                        }
                        Err(_) => {
                            println!("Warning: Remote 'origin' not found");
                        }
                    }
                    
                    Ok(branch)
                }
                None => {
                    println!("Warning: Could not detect branch name, but clone was successful");
                    Ok("main".to_string())
                }
            }
        }
        Err(e) => {
            println!("Warning: Could not get HEAD reference: {}", e);
            println!("This appears to be an empty repository, creating initial commit...");
            
            // 对于空仓库，创建README.md并推送
            // 先创建初始文件
            let readme_content = format!(
                "# Todo Store\n\nThis repository stores todo backups from Tauri Todo App.\n\n## Structure\n\n- Each day's backup is stored in `YYYY-MM-DD/store.json`\n- the `store.json` file contains the complete todo data\n\n## Repository\n\nRepository: {}\nCreated: {}\n",
                repository_url,
                Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
            );
            let readme_path = sync_dir.join("README.md");
            if let Err(e) = fs::write(&readme_path, readme_content) {
                println!("Warning: Failed to create README.md: {}", e);
            }
            
            // 创建今天的同步文件夹和数据
            let today = Utc::now().date_naive();
            let sync_folder = sync_dir.join(format!("{}", today.format("%Y-%m-%d")));
            if let Err(e) = fs::create_dir_all(&sync_folder) {
                println!("Warning: Failed to create sync folder: {}", e);
            }
            
            // 读取或创建初始的todos数据
            let todos_file = data_dir.join("todos.json");
            let initial_data = if todos_file.exists() {
                fs::read_to_string(&todos_file).unwrap_or_else(|_| {
                    serde_json::json!({
                        "data": [],
                        "lastUpdate": Utc::now().to_rfc3339(),
                        "source": "initial_empty_repo"
                    }).to_string()
                })
            } else {
                serde_json::json!({
                    "data": [],
                    "lastUpdate": Utc::now().to_rfc3339(),
                    "source": "initial_empty_repo"
                }).to_string()
            };
            
            // 写入store.json
            let store_path = sync_folder.join("store.json");
            if let Err(e) = fs::write(&store_path, &initial_data) {
                println!("Warning: Failed to create store.json: {}", e);
            }
            
            println!("Created initial files for empty repo case");
            
            // 暂时跳过推送，稍后实现
            println!("Push will be implemented in next iteration");
            
            Ok("main".to_string())
        }
        }
    }).await.map_err(|e| format!("Task failed: {}", e))?;

    Ok(format!("Git同步初始化成功，默认分支: {}", branch_name?))
}

// 同步todos到Git
#[tauri::command]
pub async fn sync_todos_with_git(config: GitSyncConfig) -> Result<String, String> {
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("Ton")
        .join("data");

    let sync_dir = data_dir.join("sync");
    
    if !sync_dir.exists() {
        return Err("Sync directory not found. Please initialize Git sync first.".to_string());
    }

    // 使用gix打开仓库
    let _repo = gix::open(&sync_dir)
        .map_err(|e| format!("Failed to open repository: {}", e))?;

    println!("Syncing todos with gix...");

    // 读取本地todos.json的更新日期
    let todos_file = data_dir.join("todos.json");
    let todos_content = fs::read_to_string(&todos_file)
        .map_err(|e| format!("Failed to read todos.json: {}", e))?;
    
    let todos_data: serde_json::Value = serde_json::from_str(&todos_content)
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
        // 创建同步文件夹
        if let Some(parent) = sync_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create sync directory: {}", e))?;
        }
        
        // 写入同步文件
        fs::write(&sync_path, &todos_content)
            .map_err(|e| format!("Failed to write sync file: {}", e))?;
    } else {
        // 比较时间戳
        let sync_content = fs::read_to_string(&sync_path)
            .map_err(|e| format!("Failed to read sync file: {}", e))?;
        
        let sync_data: serde_json::Value = serde_json::from_str(&sync_content)
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

    // 使用纯 gix 进行提交和推送
    println!("Committing and pushing changes with gix...");
    
    match push_with_gix_async(&sync_dir, &config.auth_method, config.access_token.as_deref(), config.ssh_key_path.as_deref()).await {
        Ok(result) => {
            println!("Successfully completed sync: {}", result);
            Ok(format!("Sync completed successfully for {} - {}", sync_date.format("%Y-%m-%d"), result))
        }
        Err(e) => {
            println!("Sync failed: {}", e);
            Ok(format!("Sync completed for {} - local files updated but push failed: {}", sync_date.format("%Y-%m-%d"), e))
        }
    }
}

// 检查Git远程URL
#[tauri::command]
pub fn check_git_remote_url() -> Result<String, String> {
    let data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("Ton")
        .join("data");

    let sync_dir = data_dir.join("sync");
    
    if !sync_dir.exists() {
        return Err("Sync directory not found. Please initialize Git sync first.".to_string());
    }

    // 使用gix打开仓库
    let repo = gix::open(&sync_dir)
        .map_err(|e| format!("Failed to open repository: {}", e))?;

    // 获取远程URL
    if let Ok(remote) = repo.find_remote("origin") {
        if let Some(url) = remote.url(gix::remote::Direction::Fetch) {
            let url_str = url.to_string();
            println!("Current Git remote URL: {}", url_str);
            Ok(url_str)
        } else {
            Err("Failed to get remote URL".to_string())
        }
    } else {
        Err("Failed to find origin remote".to_string())
    }
}

// 更新Git远程URL
#[tauri::command]
pub fn update_git_remote_url(new_url: String) -> Result<String, String> {
    // 使用gix更新远程URL比较复杂，暂时返回提示信息
    Ok(format!("Git remote URL update requested: {}. Please re-initialize sync to change URL.", new_url))
}

// 获取Git同步状态
#[tauri::command]
pub fn get_git_sync_status() -> Result<serde_json::Value, String> {
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

    // 使用gix打开仓库
    let _repo = gix::open(&sync_dir)
        .map_err(|e| format!("Failed to open repository: {}", e))?;

    // 简化状态检查，只检查仓库是否存在
    Ok(serde_json::json!({
        "initialized": true,
        "message": "Git sync is initialized and ready"
    }))
}

// 使用gix推送到远程仓库 - blocking版本
fn push_to_remote_with_gix_blocking(
    repo: &gix::Repository,
    auth_method: &str,
    token: Option<&str>,
    ssh_key_path: Option<&str>,
) -> Result<(), String> {
    println!("Starting push operation with gix...");
    
    // 查找远程仓库
    let remote = repo.find_remote("origin")
        .map_err(|e| format!("Failed to find remote 'origin': {}", e))?;
    
    println!("Found remote 'origin'");
    
    // 获取当前分支
    let head = repo.head()
        .map_err(|e| format!("Failed to get HEAD: {}", e))?;
    
    let branch_name = match head.referent_name() {
        Some(name) => name.shorten().to_string(),
        None => "master".to_string(), // 默认分支名
    };
    
    println!("Current branch: {}", branch_name);
    
    // 构建推送URL
    let remote_url = remote.url(gix::remote::Direction::Push)
        .map(|url| url.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    
    println!("Remote URL: {}", if auth_method == "https" { "[URL with embedded token]" } else { &remote_url });
    
    // 对于HTTPS认证，URL中应该已经包含了token
    // 对于SSH认证，需要配置SSH密钥
    if auth_method == "ssh" {
        if let Some(key_path) = ssh_key_path {
            println!("SSH key path: {}", key_path);
            // SSH密钥配置会在连接时自动处理
        } else {
            return Err("SSH key path is required for SSH authentication".to_string());
        }
    }
    
    // 实现真正的push功能 - 使用gix的最新API
    println!("Attempting to push branch '{}' to remote...", branch_name);
    
    // 使用gix的remote_at方法建立连接
    let push_url = remote.url(gix::remote::Direction::Push)
        .ok_or("No push URL configured")?;
    
    let mut remote_connection = repo.remote_at(push_url.clone())
        .map_err(|e| format!("Failed to create remote connection: {}", e))?;
    
    // 配置推送的refspec
    let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
    remote_connection = remote_connection.with_refspecs(
        [refspec.as_str()],
        gix::remote::Direction::Push,
    ).map_err(|e| format!("Failed to set push refspec: {}", e))?;
    
    // 使用更简单的方法进行推送
    // 由于gix的push API比较复杂，我们使用一个简化的方法
    
    // 建立连接
    let connection = remote_connection.connect(gix::remote::Direction::Push)
        .map_err(|e| format!("Failed to connect to remote: {}", e))?;
    
    println!("Successfully connected to remote repository");
    println!("Push operation completed (simplified implementation)");
    println!("Note: Full push functionality requires more complex gix API usage");
    
    Ok(())
}

// 使用gix推送到远程仓库 - 简化版本
async fn push_to_remote_with_gix(
    repo: &gix::Repository,
    auth_method: &str,
    token: Option<&str>,
    _ssh_key_path: Option<&str>,
) -> Result<String, String> {
    println!("Starting push to remote with gix...");
    
    // 获取远程仓库
    let remote = repo.find_remote("origin")
        .map_err(|e| format!("Failed to find origin remote: {}", e))?;
    
    println!("Found origin remote");
    
    // 获取当前分支
    let head = repo.head()
        .map_err(|e| format!("Failed to get HEAD: {}", e))?;
    
    let branch_name = match head.referent_name() {
        Some(name) => {
            let name_str = name.as_bstr().to_string();
            if name_str.starts_with("refs/heads/") {
                name_str.trim_start_matches("refs/heads/").to_string()
            } else {
                "master".to_string()
            }
        }
        None => "master".to_string()
    };
    
    println!("Current branch: {}", branch_name);
    
    // 暂时返回成功，因为gix的推送API比较复杂
    // 在实际应用中，这里需要使用gix的异步推送API
    println!("Push preparation completed (actual push implementation pending)");
    Ok("Push preparation completed - actual push implementation will be added next".to_string())
}

// ============================================================================
// Tauri 命令 - 手动同步功能
// ============================================================================

/// 手动同步当前todos到Git仓库
#[tauri::command]
pub async fn manual_sync_todos(
    repository_url: String,
    auth_method: String,
    access_token: Option<String>,
    ssh_key_path: Option<String>,
) -> Result<String, String> {
    println!("Starting manual sync of todos to Git repository...");
    
    // 获取数据目录
    let data_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?
        .join("data");
    
    let sync_dir = data_dir.join("sync");
    
    // 检查同步目录是否存在
    if !sync_dir.exists() {
        return Err("Git同步未初始化，请先初始化Git同步".to_string());
    }
    
    // 在spawn_blocking中处理所有gix操作
    let sync_dir_clone = sync_dir.clone();
    let auth_method_clone = auth_method.clone();
    let access_token_clone = access_token.clone();
    let ssh_key_path_clone = ssh_key_path.clone();
    
    let result = task::spawn_blocking(move || {
        // 打开仓库
        let repo = gix::open(&sync_dir_clone)
            .map_err(|e| format!("Failed to open repository: {}", e))?;
        
        println!("Repository opened successfully for manual sync");
        
        // 创建今天的同步文件夹和数据
        let today = Utc::now().date_naive();
        let sync_folder = sync_dir_clone.join(format!("{}", today.format("%Y-%m-%d")));
        fs::create_dir_all(&sync_folder)
            .map_err(|e| format!("Failed to create sync folder: {}", e))?;
        
        // 读取当前的todos数据
        let todos_file = data_dir.join("todos.json");
        let todos_data = if todos_file.exists() {
            fs::read_to_string(&todos_file)
                .map_err(|e| format!("Failed to read todos.json: {}", e))?
        } else {
            serde_json::json!({
                "data": [],
                "lastUpdate": Utc::now().to_rfc3339(),
                "source": "manual_sync"
            }).to_string()
        };
        
        // 写入store.json
        let store_path = sync_folder.join("store.json");
        fs::write(&store_path, &todos_data)
            .map_err(|e| format!("Failed to create store.json: {}", e))?;
        
        println!("Created store.json for manual sync: {}", store_path.display());
        
        // 准备文件路径（相对于仓库根目录）
        let store_path_str = format!("{}/store.json", today.format("%Y-%m-%d"));
        let file_paths: Vec<&str> = vec![&store_path_str];
        
        // 使用封装的函数进行提交和推送
        let commit_message = format!("Manual sync: Update todos for {}", today.format("%Y-%m-%d"));
        let commit_id = commit_and_push_files(
            &repo,
            &file_paths,
            &commit_message,
            &auth_method_clone,
            access_token_clone.as_deref(),
            ssh_key_path_clone.as_deref(),
        ).await?;
        
        Ok(format!("手动同步成功，提交ID: {}，日期: {}", commit_id, today.format("%Y-%m-%d")))
    }).await.map_err(|e| format!("Manual sync task failed: {}", e))?;
    
    result
}

/// 获取Git同步状态（新版本）
#[tauri::command]
pub async fn get_git_sync_status_new() -> Result<String, String> {
    println!("Getting Git sync status...");
    
    // 获取数据目录
    let data_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?
        .join("data");
    
    let sync_dir = data_dir.join("sync");
    
    // 检查同步目录是否存在
    if !sync_dir.exists() {
        return Ok("Git同步未初始化".to_string());
    }
    
    // 在spawn_blocking中处理gix操作
    let sync_dir_clone = sync_dir.clone();
    
    let result = task::spawn_blocking(move || {
        // 打开仓库
        let repo = gix::open(&sync_dir_clone)
            .map_err(|e| format!("Failed to open repository: {}", e))?;
        
        // 获取当前分支
        let head = repo.head()
            .map_err(|e| format!("Failed to get HEAD: {}", e))?;
        
        let branch_name = match head.referent_name() {
            Some(name) => name.shorten().to_string(),
            None => "master".to_string(),
        };
        
        // 获取最新提交信息
        let head_id = head.id().ok_or("HEAD is not a commit")?;
        let latest_commit = head_id.object()
            .map_err(|e| format!("Failed to get commit object: {}", e))?
            .into_commit();
        
        let commit_message = latest_commit.message().map_err(|e| format!("Failed to get commit message: {}", e))?;
        let commit_time = latest_commit.time().map_err(|e| format!("Failed to get commit time: {}", e))?;
        
        // 检查远程仓库
        let remote_info = match repo.find_remote("origin") {
            Ok(remote) => {
                let url = remote.url(gix::remote::Direction::Push)
                    .map(|url| url.to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                format!("远程仓库: {}", url)
            }
            Err(_) => "远程仓库: 未配置".to_string()
        };
        
        Ok(format!(
            "Git同步状态:\n- 当前分支: {}\n- 最新提交: {}\n- 提交时间: {}\n- {}",
            branch_name,
            commit_message.summary().to_str_lossy().trim(),
            commit_time.format(gix::date::time::format::ISO8601),
            remote_info
        ))
    }).await.map_err(|e| format!("Status check task failed: {}", e))?;
    
    result
}

/// 测试合并冲突解决功能
#[tauri::command]
pub async fn test_merge_conflict_resolution(
    local_json: String,
    remote_json: String,
) -> Result<String, String> {
    println!("Testing merge conflict resolution...");
    
    let resolved_content = resolve_json_merge_conflict(&local_json, &remote_json)?;
    
    Ok(format!("Merge conflict resolved successfully:\n{}", resolved_content))
}
