use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;
use tokio::sync::Mutex;
use sqlx::{MySqlPool, Row};
use std::sync::Arc;

// 导入数据模块的函数
use crate::modules::data::{save_todos, load_todos, save_settings, load_settings};

// 数据库配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

// 同步状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub is_connected: bool,
    pub is_syncing: bool,
    pub last_sync_time: Option<String>,
    pub error: Option<String>,
}

// 同步结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub message: String,
    pub data: Option<SyncData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncData {
    pub local_last_update: String,
    pub remote_last_update: String,
    pub synced_items: usize,
}

// 数据库连接池状态
pub struct DatabaseState {
    pub pool: Arc<Mutex<Option<MySqlPool>>>,
    pub config: Arc<Mutex<Option<DatabaseConfig>>>,
}

impl Default for DatabaseState {
    fn default() -> Self {
        Self {
            pool: Arc::new(Mutex::new(None)),
            config: Arc::new(Mutex::new(None)),
        }
    }
}

// 注意：加密功能暂时移除，配置以明文存储
// 在实际部署时应该实现真正的加密存储

// 获取数据目录路径
fn get_data_dir() -> Result<std::path::PathBuf, String> {
    let data_dir = dirs::data_dir()
        .ok_or("无法获取数据目录")?
        .join("tauri-todos");
    
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("创建数据目录失败: {}", e))?;
    
    Ok(data_dir)
}

// 保存数据库配置（暂时不加密）
#[tauri::command]
pub async fn save_database_config(config: DatabaseConfig) -> Result<(), String> {
    let data_dir = get_data_dir()?;
    let config_path = data_dir.join("database_config.json");
    
    // 序列化配置
    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    
    // 保存到文件
    std::fs::write(&config_path, config_json)
        .map_err(|e| format!("保存配置文件失败: {}", e))?;
    
    Ok(())
}

// 加载数据库配置
#[tauri::command]
pub async fn load_database_config() -> Result<Option<DatabaseConfig>, String> {
    let data_dir = get_data_dir()?;
    let config_path = data_dir.join("database_config.json");
    
    if !config_path.exists() {
        return Ok(None);
    }
    
    // 读取文件
    let config_data = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;
    
    // 反序列化配置
    let config: DatabaseConfig = serde_json::from_str(&config_data)
        .map_err(|e| format!("反序列化配置失败: {}", e))?;
    
    Ok(Some(config))
}

// 测试数据库连接
#[tauri::command]
pub async fn test_database_connection(config: DatabaseConfig) -> Result<bool, String> {
    let connection_string = format!(
        "mysql://{}:{}@{}:{}/{}",
        config.username, config.password, config.host, config.port, config.database
    );
    
    match MySqlPool::connect(&connection_string).await {
        Ok(pool) => {
            // 测试查询
            match sqlx::query("SELECT 1").fetch_one(&pool).await {
                Ok(_) => Ok(true),
                Err(e) => Err(format!("数据库查询测试失败: {}", e)),
            }
        }
        Err(e) => Err(format!("数据库连接失败: {}", e)),
    }
}

// 创建数据库连接池
async fn create_pool(config: &DatabaseConfig) -> Result<MySqlPool, String> {
    let connection_string = format!(
        "mysql://{}:{}@{}:{}/{}",
        config.username, config.password, config.host, config.port, config.database
    );
    
    MySqlPool::connect(&connection_string).await
        .map_err(|e| format!("创建数据库连接池失败: {}", e))
}

// 检查表是否存在
async fn table_exists(pool: &MySqlPool, table_name: &str) -> Result<bool, String> {
    let query = "SELECT COUNT(*) as count FROM information_schema.tables WHERE table_schema = DATABASE() AND table_name = ?";
    
    match sqlx::query(query)
        .bind(table_name)
        .fetch_one(pool)
        .await
    {
        Ok(row) => {
            let count: i64 = row.get("count");
            Ok(count > 0)
        }
        Err(e) => Err(format!("检查表存在性失败: {}", e)),
    }
}

// 检查表结构是否匹配
async fn check_table_structure(pool: &MySqlPool, table_name: &str, expected_columns: &[&str]) -> Result<bool, String> {
    let query = r#"
        SELECT COLUMN_NAME 
        FROM information_schema.columns 
        WHERE table_schema = DATABASE() AND table_name = ?
        ORDER BY ORDINAL_POSITION
    "#;
    
    let rows = sqlx::query(query)
        .bind(table_name)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("检查表结构失败: {}", e))?;
    
    let existing_columns: Vec<String> = rows.iter()
        .map(|row| row.get::<String, _>("COLUMN_NAME"))
        .collect();
    
    // 检查是否包含所有必需的列
    for expected_col in expected_columns {
        if !existing_columns.contains(&expected_col.to_string()) {
            return Ok(false);
        }
    }
    
    Ok(true)
}

// 修改表结构以匹配期望的结构
async fn alter_table_structure(pool: &MySqlPool, table_name: &str) -> Result<(), String> {
    match table_name {
        "todo_settings_sync" => {
            // 检查并添加缺失的列
            let alter_queries = vec![
                "ALTER TABLE todo_settings_sync ADD COLUMN IF NOT EXISTS id INT AUTO_INCREMENT PRIMARY KEY FIRST",
                "ALTER TABLE todo_settings_sync ADD COLUMN IF NOT EXISTS update_time VARCHAR(50) NOT NULL COMMENT '更新时间'",
                "ALTER TABLE todo_settings_sync ADD COLUMN IF NOT EXISTS field_name VARCHAR(100) NOT NULL COMMENT '字段名'",
                "ALTER TABLE todo_settings_sync ADD COLUMN IF NOT EXISTS data_type VARCHAR(50) NOT NULL COMMENT '数据类型'",
                "ALTER TABLE todo_settings_sync ADD COLUMN IF NOT EXISTS field_value TEXT NOT NULL COMMENT '字段值'",
                "ALTER TABLE todo_settings_sync ADD COLUMN IF NOT EXISTS last_update VARCHAR(50) NOT NULL COMMENT '最后更新时间'",
                "ALTER TABLE todo_settings_sync ADD COLUMN IF NOT EXISTS created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP",
                "ALTER TABLE todo_settings_sync ADD COLUMN IF NOT EXISTS updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP",
            ];
            
            for query in alter_queries {
                if let Err(e) = sqlx::query(query).execute(pool).await {
                    // 忽略列已存在的错误
                    if !e.to_string().contains("Duplicate column name") {
                        return Err(format!("修改表结构失败: {}", e));
                    }
                }
            }
        }
        "todo_items_sync" => {
            let alter_queries = vec![
                "ALTER TABLE todo_items_sync ADD COLUMN IF NOT EXISTS id VARCHAR(50) PRIMARY KEY COMMENT '待办事项ID'",
                "ALTER TABLE todo_items_sync ADD COLUMN IF NOT EXISTS parent_id VARCHAR(50) NULL COMMENT '父项ID，支持树形结构'",
                "ALTER TABLE todo_items_sync ADD COLUMN IF NOT EXISTS text TEXT NOT NULL COMMENT '待办事项内容'",
                "ALTER TABLE todo_items_sync ADD COLUMN IF NOT EXISTS completed BOOLEAN NOT NULL DEFAULT FALSE COMMENT '是否完成'",
                "ALTER TABLE todo_items_sync ADD COLUMN IF NOT EXISTS created_at VARCHAR(50) NOT NULL COMMENT '创建时间'",
                "ALTER TABLE todo_items_sync ADD COLUMN IF NOT EXISTS completed_at VARCHAR(50) NULL COMMENT '完成时间'",
                "ALTER TABLE todo_items_sync ADD COLUMN IF NOT EXISTS deadline VARCHAR(50) NULL COMMENT '截止时间'",
                "ALTER TABLE todo_items_sync ADD COLUMN IF NOT EXISTS last_update VARCHAR(50) NOT NULL COMMENT '最后更新时间'",
                "ALTER TABLE todo_items_sync ADD COLUMN IF NOT EXISTS created_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP",
                "ALTER TABLE todo_items_sync ADD COLUMN IF NOT EXISTS updated_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP",
            ];
            
            for query in alter_queries {
                if let Err(e) = sqlx::query(query).execute(pool).await {
                    // 忽略列已存在的错误
                    if !e.to_string().contains("Duplicate column name") {
                        return Err(format!("修改表结构失败: {}", e));
                    }
                }
            }
        }
        _ => return Err(format!("未知的表名: {}", table_name)),
    }
    
    Ok(())
}

// 检查并初始化数据库表结构
#[tauri::command]
pub async fn check_and_initialize_tables(
    state: State<'_, DatabaseState>
) -> Result<String, String> {
    let pool_guard = state.pool.lock().await;
    let pool = pool_guard.as_ref()
        .ok_or("数据库连接未建立")?;
    
    let mut messages = Vec::new();
    
    // 检查设置同步表
    let settings_table_exists = table_exists(pool, "todo_settings_sync").await?;
    if !settings_table_exists {
        create_settings_sync_table(pool).await?;
        messages.push("创建了设置同步表");
    } else {
        let expected_columns = ["id", "update_time", "field_name", "data_type", "field_value", "last_update"];
        let structure_matches = check_table_structure(pool, "todo_settings_sync", &expected_columns).await?;
        if !structure_matches {
            alter_table_structure(pool, "todo_settings_sync").await?;
            messages.push("更新了设置同步表结构");
        } else {
            messages.push("设置同步表结构正常");
        }
    }
    
    // 检查待办同步表
    let todos_table_exists = table_exists(pool, "todo_items_sync").await?;
    if !todos_table_exists {
        create_todos_sync_table(pool).await?;
        messages.push("创建了待办同步表");
    } else {
        let expected_columns = ["id", "parent_id", "text", "completed", "created_at", "completed_at", "deadline", "last_update"];
        let structure_matches = check_table_structure(pool, "todo_items_sync", &expected_columns).await?;
        if !structure_matches {
            alter_table_structure(pool, "todo_items_sync").await?;
            messages.push("更新了待办同步表结构");
        } else {
            messages.push("待办同步表结构正常");
        }
    }
    
    Ok(messages.join("；"))
}

// 创建设置同步表
async fn create_settings_sync_table(pool: &MySqlPool) -> Result<(), String> {
    let create_table_sql = r#"
        CREATE TABLE IF NOT EXISTS todo_settings_sync (
            id INT AUTO_INCREMENT PRIMARY KEY,
            update_time VARCHAR(50) NOT NULL COMMENT '更新时间',
            field_name VARCHAR(100) NOT NULL COMMENT '字段名',
            data_type VARCHAR(50) NOT NULL COMMENT '数据类型',
            field_value TEXT NOT NULL COMMENT '字段值',
            last_update VARCHAR(50) NOT NULL COMMENT '最后更新时间',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            INDEX idx_field_name (field_name),
            INDEX idx_last_update (last_update)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
    "#;
    
    sqlx::query(create_table_sql)
        .execute(pool)
        .await
        .map_err(|e| format!("创建设置同步表失败: {}", e))?;
    
    Ok(())
}

// 创建待办同步表
async fn create_todos_sync_table(pool: &MySqlPool) -> Result<(), String> {
    let create_table_sql = r#"
        CREATE TABLE IF NOT EXISTS todo_items_sync (
            id VARCHAR(50) PRIMARY KEY COMMENT '待办事项ID',
            parent_id VARCHAR(50) NULL COMMENT '父项ID，支持树形结构',
            text TEXT NOT NULL COMMENT '待办事项内容',
            completed BOOLEAN NOT NULL DEFAULT FALSE COMMENT '是否完成',
            created_at VARCHAR(50) NOT NULL COMMENT '创建时间',
            completed_at VARCHAR(50) NULL COMMENT '完成时间',
            deadline VARCHAR(50) NULL COMMENT '截止时间',
            last_update VARCHAR(50) NOT NULL COMMENT '最后更新时间',
            created_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            INDEX idx_parent_id (parent_id),
            INDEX idx_completed (completed),
            INDEX idx_last_update (last_update),
            INDEX idx_deadline (deadline)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
    "#;
    
    sqlx::query(create_table_sql)
        .execute(pool)
        .await
        .map_err(|e| format!("创建待办同步表失败: {}", e))?;
    
    Ok(())
}

// 注意：initialize_database_tables 函数已被 check_and_initialize_tables 替代

// 建立数据库连接
#[tauri::command]
pub async fn connect_database(
    config: DatabaseConfig,
    state: State<'_, DatabaseState>
) -> Result<(), String> {
    let pool = create_pool(&config).await?;
    
    // 保存连接池和配置到状态
    {
        let mut pool_guard = state.pool.lock().await;
        *pool_guard = Some(pool);
    }
    
    {
        let mut config_guard = state.config.lock().await;
        *config_guard = Some(config);
    }
    
    Ok(())
}

// 获取远程最后更新时间
async fn get_remote_last_update(pool: &MySqlPool) -> Result<String, String> {
    // 从设置表中获取 last_update
    let query = "SELECT field_value FROM todo_settings_sync WHERE field_name = 'last_update' ORDER BY id DESC LIMIT 1";
    
    match sqlx::query(query)
        .fetch_optional(pool)
        .await
    {
        Ok(Some(row)) => {
            let value: String = row.get("field_value");
            Ok(value)
        }
        Ok(None) => {
            // 如果没有记录，返回当前时间
            Ok(chrono::Utc::now().to_rfc3339())
        }
        Err(e) => Err(format!("获取远程最后更新时间失败: {}", e)),
    }
}

// 同步设置数据
async fn sync_settings_data(
    pool: &MySqlPool,
    local_settings: &Value,
    local_last_update: &str
) -> Result<usize, String> {
    let mut synced_count = 0;
    
    // 将设置对象转换为键值对
    if let Some(settings_obj) = local_settings.as_object() {
        for (key, value) in settings_obj {
            let field_value = serde_json::to_string(value)
                .map_err(|e| format!("序列化设置值失败: {}", e))?;
            
            let data_type = match value {
                Value::String(_) => "string",
                Value::Number(_) => "number",
                Value::Bool(_) => "boolean",
                Value::Object(_) => "object",
                Value::Array(_) => "array",
                Value::Null => "null",
            };
            
            // 插入或更新设置
            let query = r#"
                INSERT INTO todo_settings_sync (update_time, field_name, data_type, field_value, last_update)
                VALUES (?, ?, ?, ?, ?)
                ON DUPLICATE KEY UPDATE
                    update_time = VALUES(update_time),
                    data_type = VALUES(data_type),
                    field_value = VALUES(field_value),
                    last_update = VALUES(last_update)
            "#;
            
            sqlx::query(query)
                .bind(local_last_update)
                .bind(key)
                .bind(data_type)
                .bind(&field_value)
                .bind(local_last_update)
                .execute(pool)
                .await
                .map_err(|e| format!("同步设置数据失败: {}", e))?;
            
            synced_count += 1;
        }
    }
    
    Ok(synced_count)
}

// 同步待办数据
async fn sync_todos_data(
    pool: &MySqlPool,
    local_todos: &[Value],
    local_last_update: &str
) -> Result<usize, String> {
    let mut synced_count = 0;
    
    for todo in local_todos {
        if let Some(todo_obj) = todo.as_object() {
            let id = todo_obj.get("id")
                .and_then(|v| v.as_str())
                .ok_or("待办事项缺少ID")?;
            
            let text = todo_obj.get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            let completed = todo_obj.get("completed")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            
            let created_at = todo_obj.get("createdAt")
                .and_then(|v| v.as_str())
                .unwrap_or(&local_last_update);
            
            let completed_at = todo_obj.get("completedAt")
                .and_then(|v| v.as_str());
            
            let deadline = todo_obj.get("deadline")
                .and_then(|v| v.as_str());
            
            let parent_id = todo_obj.get("parentId")
                .and_then(|v| v.as_str());
            
            // 插入或更新待办事项
            let query = r#"
                INSERT INTO todo_items_sync (id, parent_id, text, completed, created_at, completed_at, deadline, last_update)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                ON DUPLICATE KEY UPDATE
                    parent_id = VALUES(parent_id),
                    text = VALUES(text),
                    completed = VALUES(completed),
                    created_at = VALUES(created_at),
                    completed_at = VALUES(completed_at),
                    deadline = VALUES(deadline),
                    last_update = VALUES(last_update)
            "#;
            
            sqlx::query(query)
                .bind(id)
                .bind(parent_id)
                .bind(text)
                .bind(completed)
                .bind(created_at)
                .bind(completed_at)
                .bind(deadline)
                .bind(local_last_update)
                .execute(pool)
                .await
                .map_err(|e| format!("同步待办数据失败: {}", e))?;
            
            synced_count += 1;
        }
    }
    
    Ok(synced_count)
}

// 从远程下载设置数据
async fn download_settings_data(pool: &MySqlPool) -> Result<Value, String> {
    let query = "SELECT field_name, field_value FROM todo_settings_sync WHERE field_name != 'last_update'";
    
    let rows = sqlx::query(query)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("下载设置数据失败: {}", e))?;
    
    let mut settings = serde_json::Map::new();
    
    for row in rows {
        let field_name: String = row.get("field_name");
        let field_value: String = row.get("field_value");
        
        // 尝试解析 JSON 值
        let parsed_value: Value = serde_json::from_str(&field_value)
            .unwrap_or(Value::String(field_value));
        
        settings.insert(field_name, parsed_value);
    }
    
    Ok(Value::Object(settings))
}

// 从远程下载待办数据
async fn download_todos_data(pool: &MySqlPool) -> Result<Vec<Value>, String> {
    let query = r#"
        SELECT id, parent_id, text, completed, created_at, completed_at, deadline, last_update
        FROM todo_items_sync
        ORDER BY created_timestamp
    "#;
    
    let rows = sqlx::query(query)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("下载待办数据失败: {}", e))?;
    
    let mut todos = Vec::new();
    
    for row in rows {
        let mut todo = serde_json::Map::new();
        
        todo.insert("id".to_string(), Value::String(row.get::<String, _>("id")));
        
        if let Some(parent_id) = row.get::<Option<String>, _>("parent_id") {
            todo.insert("parentId".to_string(), Value::String(parent_id));
        }
        
        todo.insert("text".to_string(), Value::String(row.get::<String, _>("text")));
        todo.insert("completed".to_string(), Value::Bool(row.get::<bool, _>("completed")));
        todo.insert("createdAt".to_string(), Value::String(row.get::<String, _>("created_at")));
        
        if let Some(completed_at) = row.get::<Option<String>, _>("completed_at") {
            todo.insert("completedAt".to_string(), Value::String(completed_at));
        }
        
        if let Some(deadline) = row.get::<Option<String>, _>("deadline") {
            todo.insert("deadline".to_string(), Value::String(deadline));
        }
        
        todos.push(Value::Object(todo));
    }
    
    Ok(todos)
}

// 保存下载的数据到本地
fn save_downloaded_data(todos: &[Value], settings: &Value, last_update: &str) -> Result<(), String> {
    // 保存待办数据
    let todo_data = serde_json::json!({
        "data": todos,
        "lastUpdate": last_update,
        "source": "sync"
    });
    
    save_todos(todo_data)?;
    
    // 保存设置数据
    save_settings(settings.clone())?;
    
    Ok(())
}

// 开始数据库同步
#[tauri::command]
pub async fn start_database_sync(
    state: State<'_, DatabaseState>
) -> Result<SyncResult, String> {
    let pool_guard = state.pool.lock().await;
    let pool = pool_guard.as_ref()
        .ok_or("数据库连接未建立")?;
    
    // 获取本地数据
    let local_todos = load_todos()?;
    let local_settings = load_settings()?;
    let default_time = chrono::Utc::now().to_rfc3339();
    let local_last_update = local_todos.get("lastUpdate")
        .and_then(|v| v.as_str())
        .unwrap_or(&default_time);
    
    // 获取远程最后更新时间
    let remote_last_update = get_remote_last_update(pool).await?;
    
    // 比较时间戳决定同步方向
    let local_time = chrono::DateTime::parse_from_rfc3339(local_last_update)
        .map_err(|e| format!("解析本地时间失败: {}", e))?;
    let remote_time = chrono::DateTime::parse_from_rfc3339(&remote_last_update)
        .map_err(|e| format!("解析远程时间失败: {}", e))?;
    
    let mut synced_items = 0;
    let message = if local_time > remote_time {
        // 本地较新，上传到远程
        let settings_count = sync_settings_data(pool, &local_settings, local_last_update).await?;
        let empty_vec = vec![];
        let todos_data = local_todos.get("data")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_vec);
        let todos_count = sync_todos_data(pool, todos_data, local_last_update).await?;
        
        synced_items = settings_count + todos_count;
        format!("同步成功，已上传 {} 项数据到远程", synced_items)
    } else if remote_time > local_time {
        // 远程较新，从远程下载
        let remote_settings = download_settings_data(pool).await?;
        let remote_todos = download_todos_data(pool).await?;
        
        // 保存到本地
        save_downloaded_data(&remote_todos, &remote_settings, &remote_last_update)?;
        
        synced_items = remote_todos.len() + remote_settings.as_object().map_or(0, |obj| obj.len());
        format!("同步成功，已从远程下载 {} 项数据", synced_items)
    } else {
        "数据已是最新版本".to_string()
    };
    
    Ok(SyncResult {
        success: true,
        message,
        data: Some(SyncData {
            local_last_update: local_last_update.to_string(),
            remote_last_update,
            synced_items,
        }),
    })
}
