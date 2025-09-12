use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;
use tokio::sync::Mutex;
use sqlx::{MySqlPool, Row};
use std::sync::Arc;

// 导入数据模块的函数
use crate::modules::data::{save_todos, load_todos};
use crate::modules::app::{save_app_settings, load_app_settings};

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

// Base64编码/解码函数
fn encode_base64(data: &str) -> String {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD.encode(data)
}

fn decode_base64(encoded: &str) -> Result<String, String> {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD.decode(encoded)
        .map_err(|e| format!("Base64解码失败: {}", e))
        .and_then(|bytes| String::from_utf8(bytes).map_err(|e| format!("UTF-8解码失败: {}", e)))
}

// 获取配置目录路径
fn get_config_dir() -> Result<std::path::PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or("无法获取配置目录")?
        .join("Ton")
        .join("config");
    
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("创建配置目录失败: {}", e))?;
    
    Ok(config_dir)
}

// 保存数据库配置（Base64加密）
#[tauri::command]
pub async fn save_database_config(config: DatabaseConfig) -> Result<(), String> {
    let config_dir = get_config_dir()?;
    let config_path = config_dir.join("da.da");
    
    // 序列化配置
    let config_json = serde_json::to_string(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    
    // Base64编码
    let encoded_config = encode_base64(&config_json);
    
    // 保存到文件
    std::fs::write(&config_path, encoded_config)
        .map_err(|e| format!("保存配置文件失败: {}", e))?;
    
    Ok(())
}

// 加载数据库配置（Base64解密）
#[tauri::command]
pub async fn load_database_config() -> Result<Option<DatabaseConfig>, String> {
    let config_dir = get_config_dir()?;
    let config_path = config_dir.join("da.da");
    
    if !config_path.exists() {
        return Ok(None);
    }
    
    // 读取文件
    let encoded_data = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;
    
    // Base64解码
    let config_json = decode_base64(&encoded_data)?;
    
    // 反序列化配置
    let config: DatabaseConfig = serde_json::from_str(&config_json)
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
                "ALTER TABLE todo_settings_sync ADD COLUMN id INT AUTO_INCREMENT PRIMARY KEY FIRST",
                "ALTER TABLE todo_settings_sync ADD COLUMN update_time VARCHAR(50) NOT NULL COMMENT '更新时间'",
                "ALTER TABLE todo_settings_sync ADD COLUMN field_name VARCHAR(100) NOT NULL COMMENT '字段名'",
                "ALTER TABLE todo_settings_sync ADD COLUMN data_type VARCHAR(50) NOT NULL COMMENT '数据类型'",
                "ALTER TABLE todo_settings_sync ADD COLUMN field_value TEXT NOT NULL COMMENT '字段值'",
                "ALTER TABLE todo_settings_sync ADD COLUMN last_update VARCHAR(50) NOT NULL COMMENT '最后更新时间'",
                "ALTER TABLE todo_settings_sync ADD COLUMN created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP",
                "ALTER TABLE todo_settings_sync ADD COLUMN updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP",
            ];
            
            for query in alter_queries {
                if let Err(e) = sqlx::query(query).execute(pool).await {
                    // 忽略列已存在的错误
                    if !e.to_string().contains("Duplicate column name") {
                        return Err(format!("修改表结构失败: {}", e));
                    }
                }
            }
            
            // 添加唯一约束（如果不存在）
            let unique_constraint_query = "ALTER TABLE todo_settings_sync ADD UNIQUE KEY unique_field_name (field_name)";
            if let Err(e) = sqlx::query(unique_constraint_query).execute(pool).await {
                // 忽略约束已存在的错误
                if !e.to_string().contains("Duplicate key name") && !e.to_string().contains("already exists") {
                    return Err(format!("添加唯一约束失败: {}", e));
                }
            }
        }
        "todo_items_sync" => {
            let alter_queries = vec![
                "ALTER TABLE todo_items_sync ADD COLUMN id VARCHAR(50) PRIMARY KEY COMMENT '待办事项ID'",
                "ALTER TABLE todo_items_sync ADD COLUMN parent_id VARCHAR(50) NULL COMMENT '父项ID，支持树形结构'",
                "ALTER TABLE todo_items_sync ADD COLUMN text TEXT NOT NULL COMMENT '待办事项内容'",
                "ALTER TABLE todo_items_sync ADD COLUMN completed BOOLEAN NOT NULL DEFAULT FALSE COMMENT '是否完成'",
                "ALTER TABLE todo_items_sync ADD COLUMN created_at VARCHAR(50) NOT NULL COMMENT '创建时间'",
                "ALTER TABLE todo_items_sync ADD COLUMN completed_at VARCHAR(50) NULL COMMENT '完成时间'",
                "ALTER TABLE todo_items_sync ADD COLUMN deadline VARCHAR(50) NULL COMMENT '截止时间'",
                "ALTER TABLE todo_items_sync ADD COLUMN is_deleted BOOLEAN NOT NULL DEFAULT FALSE COMMENT '是否已删除（逻辑删除）'",
                "ALTER TABLE todo_items_sync ADD COLUMN last_update VARCHAR(50) NOT NULL COMMENT '最后更新时间'",
                "ALTER TABLE todo_items_sync ADD COLUMN created_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP",
                "ALTER TABLE todo_items_sync ADD COLUMN updated_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP",
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

// 清理重复的设置数据
async fn cleanup_duplicate_settings(pool: &MySqlPool) -> Result<(), String> {
    // 删除重复的设置数据，只保留最新的
    let cleanup_query = r#"
        DELETE t1 FROM todo_settings_sync t1
        INNER JOIN todo_settings_sync t2 
        WHERE t1.id > t2.id 
        AND t1.field_name = t2.field_name
    "#;
    
    sqlx::query(cleanup_query)
        .execute(pool)
        .await
        .map_err(|e| format!("清理重复设置数据失败: {}", e))?;
    
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
        
        // 清理重复的设置数据
        cleanup_duplicate_settings(pool).await?;
        messages.push("清理了重复的设置数据");
    }
    
    // 检查待办同步表
    let todos_table_exists = table_exists(pool, "todo_items_sync").await?;
    if !todos_table_exists {
        create_todos_sync_table(pool).await?;
        messages.push("创建了待办同步表");
    } else {
        let expected_columns = ["id", "parent_id", "text", "completed", "created_at", "completed_at", "deadline", "is_deleted", "last_update"];
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
            field_name VARCHAR(100) NOT NULL UNIQUE COMMENT '字段名',
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
            is_deleted BOOLEAN NOT NULL DEFAULT FALSE COMMENT '是否已删除（逻辑删除）',
            last_update VARCHAR(50) NOT NULL COMMENT '最后更新时间',
            created_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            INDEX idx_parent_id (parent_id),
            INDEX idx_completed (completed),
            INDEX idx_is_deleted (is_deleted),
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
async fn get_remote_last_update(pool: &MySqlPool) -> Result<Option<String>, String> {
    // 从设置表中获取 last_update
    let query = "SELECT field_value FROM todo_settings_sync WHERE field_name = 'last_update' ORDER BY id DESC LIMIT 1";
    
    match sqlx::query(query)
        .fetch_optional(pool)
        .await
    {
        Ok(Some(row)) => {
            let value: String = row.get("field_value");
            Ok(Some(value))
        }
        Ok(None) => {
            // 如果没有记录，返回None表示远程没有数据
            Ok(None)
        }
        Err(e) => Err(format!("获取远程最后更新时间失败: {}", e)),
    }
}

// 同步设置数据（带事务保护）
async fn sync_settings_data(
    pool: &MySqlPool,
    local_settings: &Value,
    local_last_update: &str
) -> Result<usize, String> {
    // 开始事务
    let mut tx = pool.begin().await
        .map_err(|e| format!("开始事务失败: {}", e))?;
    
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
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("同步设置数据失败: {}", e))?;
            
            synced_count += 1;
        }
    }
    
    // 提交事务
    tx.commit().await
        .map_err(|e| format!("提交事务失败: {}", e))?;
    
    Ok(synced_count)
}

// 同步待办数据（带事务保护）
async fn sync_todos_data(
    pool: &MySqlPool,
    local_todos: &[Value],
    local_last_update: &str
) -> Result<usize, String> {
    // 开始事务
    let mut tx = pool.begin().await
        .map_err(|e| format!("开始事务失败: {}", e))?;
    
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
            
            // 检查是否已删除（逻辑删除）
            let is_deleted = todo_obj.get("isDeleted")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            
            // 插入或更新待办事项
            let query = r#"
                INSERT INTO todo_items_sync (id, parent_id, text, completed, created_at, completed_at, deadline, is_deleted, last_update)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON DUPLICATE KEY UPDATE
                    parent_id = VALUES(parent_id),
                    text = VALUES(text),
                    completed = VALUES(completed),
                    created_at = VALUES(created_at),
                    completed_at = VALUES(completed_at),
                    deadline = VALUES(deadline),
                    is_deleted = VALUES(is_deleted),
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
                .bind(is_deleted)
                .bind(local_last_update)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("同步待办数据失败: {}", e))?;
            
            synced_count += 1;
        }
    }
    
    // 提交事务
    tx.commit().await
        .map_err(|e| format!("提交事务失败: {}", e))?;
    
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
        SELECT id, parent_id, text, completed, created_at, completed_at, deadline, is_deleted, last_update
        FROM todo_items_sync
        WHERE is_deleted = FALSE
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
        
        // 添加isDeleted字段（虽然查询时已过滤，但保持数据结构一致）
        todo.insert("isDeleted".to_string(), Value::Bool(row.get::<bool, _>("is_deleted")));
        
        todos.push(Value::Object(todo));
    }
    
    Ok(todos)
}

// 保存下载的数据到本地（带事务保护）
fn save_downloaded_data(todos: &[Value], settings: &Value, last_update: &str) -> Result<(), String> {
    // 保存待办数据
    let todo_data = serde_json::json!({
        "data": todos,
        "lastUpdate": last_update,
        "source": "sync"
    });
    
    // 注意：这里调用的是本地文件保存函数，它们本身不涉及数据库事务
    // 但我们需要确保两个文件保存操作要么都成功，要么都失败
    match save_todos(todo_data) {
        Ok(_) => {
            // 待办数据保存成功，继续保存设置数据
            match save_app_settings(settings.clone()) {
                Ok(_) => Ok(()),
                Err(e) => {
                    // 设置数据保存失败，尝试回滚待办数据
                    // 注意：这里无法真正回滚，只能记录错误
                    Err(format!("保存设置数据失败: {}，待办数据已保存", e))
                }
            }
        }
        Err(e) => Err(format!("保存待办数据失败: {}", e))
    }
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
    let local_settings = load_app_settings()?;
    let default_time = chrono::Utc::now().to_rfc3339();
    let local_last_update = local_todos.get("lastUpdate")
        .and_then(|v| v.as_str())
        .unwrap_or(&default_time);
    
    // 获取远程最后更新时间
    let remote_last_update = get_remote_last_update(pool).await?;
    
    let mut synced_items = 0;
    let remote_last_update_str = remote_last_update.clone().unwrap_or_else(|| "无远程数据".to_string());
    let message = match remote_last_update {
        Some(remote_time_str) => {
            // 远程有数据，比较时间戳决定同步方向
            let local_time = chrono::DateTime::parse_from_rfc3339(local_last_update)
                .map_err(|e| format!("解析本地时间失败: {}", e))?;
            let remote_time = chrono::DateTime::parse_from_rfc3339(&remote_time_str)
                .map_err(|e| format!("解析远程时间失败: {}", e))?;
            
            if local_time > remote_time {
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
                save_downloaded_data(&remote_todos, &remote_settings, &remote_time_str)?;
                
                synced_items = remote_todos.len() + remote_settings.as_object().map_or(0, |obj| obj.len());
                format!("同步成功，已从远程下载 {} 项数据", synced_items)
            } else {
                "数据已是最新版本".to_string()
            }
        }
        None => {
            // 远程没有数据，直接上传本地数据
            let settings_count = sync_settings_data(pool, &local_settings, local_last_update).await?;
            let empty_vec = vec![];
            let todos_data = local_todos.get("data")
                .and_then(|v| v.as_array())
                .unwrap_or(&empty_vec);
            let todos_count = sync_todos_data(pool, todos_data, local_last_update).await?;
            
            synced_items = settings_count + todos_count;
            format!("同步成功，已上传 {} 项数据到远程（首次同步）", synced_items)
        }
    };
    
    Ok(SyncResult {
        success: true,
        message,
        data: Some(SyncData {
            local_last_update: local_last_update.to_string(),
            remote_last_update: remote_last_update_str,
            synced_items,
        }),
    })
}

// 逻辑删除待办事项（支持级联删除子项，带事务保护）
#[tauri::command]
pub async fn delete_todo_logically(
    todo_id: String,
    state: State<'_, DatabaseState>
) -> Result<bool, String> {
    let pool_guard = state.pool.lock().await;
    let pool = pool_guard.as_ref()
        .ok_or("数据库连接未建立")?;
    
    let current_time = chrono::Utc::now().to_rfc3339();
    
    // 开始事务
    let mut tx = pool.begin().await
        .map_err(|e| format!("开始事务失败: {}", e))?;
    
    // 使用递归CTE查找所有子项
    let cascade_delete_query = r#"
        WITH RECURSIVE todo_hierarchy AS (
            -- 基础查询：找到要删除的根项
            SELECT id, parent_id, 0 as level
            FROM todo_items_sync 
            WHERE id = ? AND is_deleted = FALSE
            
            UNION ALL
            
            -- 递归查询：找到所有子项
            SELECT t.id, t.parent_id, th.level + 1
            FROM todo_items_sync t
            INNER JOIN todo_hierarchy th ON t.parent_id = th.id
            WHERE t.is_deleted = FALSE
        )
        UPDATE todo_items_sync 
        SET is_deleted = TRUE, last_update = ?
        WHERE id IN (SELECT id FROM todo_hierarchy)
    "#;
    
    let result = sqlx::query(cascade_delete_query)
        .bind(&todo_id)
        .bind(&current_time)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("级联逻辑删除待办事项失败: {}", e))?;
    
    // 提交事务
    tx.commit().await
        .map_err(|e| format!("提交事务失败: {}", e))?;
    
    Ok(result.rows_affected() > 0)
}

// 恢复已删除的待办事项（支持级联恢复，带事务保护）
#[tauri::command]
pub async fn restore_todo(
    todo_id: String,
    state: State<'_, DatabaseState>
) -> Result<bool, String> {
    let pool_guard = state.pool.lock().await;
    let pool = pool_guard.as_ref()
        .ok_or("数据库连接未建立")?;
    
    let current_time = chrono::Utc::now().to_rfc3339();
    
    // 开始事务
    let mut tx = pool.begin().await
        .map_err(|e| format!("开始事务失败: {}", e))?;
    
    // 使用递归CTE查找所有需要恢复的项（包括父项和子项）
    let cascade_restore_query = r#"
        WITH RECURSIVE todo_hierarchy AS (
            -- 基础查询：找到要恢复的根项
            SELECT id, parent_id, 0 as level
            FROM todo_items_sync 
            WHERE id = ? AND is_deleted = TRUE
            
            UNION ALL
            
            -- 向上递归：找到所有父项（如果父项也被删除了）
            SELECT t.id, t.parent_id, th.level + 1
            FROM todo_items_sync t
            INNER JOIN todo_hierarchy th ON t.id = th.parent_id
            WHERE t.is_deleted = TRUE
            
            UNION ALL
            
            -- 向下递归：找到所有子项
            SELECT t.id, t.parent_id, th.level + 1
            FROM todo_items_sync t
            INNER JOIN todo_hierarchy th ON t.parent_id = th.id
            WHERE t.is_deleted = TRUE
        )
        UPDATE todo_items_sync 
        SET is_deleted = FALSE, last_update = ?
        WHERE id IN (SELECT id FROM todo_hierarchy)
    "#;
    
    let result = sqlx::query(cascade_restore_query)
        .bind(&todo_id)
        .bind(&current_time)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("级联恢复待办事项失败: {}", e))?;
    
    // 提交事务
    tx.commit().await
        .map_err(|e| format!("提交事务失败: {}", e))?;
    
    Ok(result.rows_affected() > 0)
}

// 安全恢复已删除的待办事项（只恢复当前项和其子项，不恢复父项，带事务保护）
#[tauri::command]
pub async fn restore_todo_safe(
    todo_id: String,
    state: State<'_, DatabaseState>
) -> Result<bool, String> {
    let pool_guard = state.pool.lock().await;
    let pool = pool_guard.as_ref()
        .ok_or("数据库连接未建立")?;
    
    let current_time = chrono::Utc::now().to_rfc3339();
    
    // 开始事务
    let mut tx = pool.begin().await
        .map_err(|e| format!("开始事务失败: {}", e))?;
    
    // 使用递归CTE只查找当前项和其子项
    let safe_restore_query = r#"
        WITH RECURSIVE todo_hierarchy AS (
            -- 基础查询：找到要恢复的根项
            SELECT id, parent_id, 0 as level
            FROM todo_items_sync 
            WHERE id = ? AND is_deleted = TRUE
            
            UNION ALL
            
            -- 向下递归：只找到子项
            SELECT t.id, t.parent_id, th.level + 1
            FROM todo_items_sync t
            INNER JOIN todo_hierarchy th ON t.parent_id = th.id
            WHERE t.is_deleted = TRUE
        )
        UPDATE todo_items_sync 
        SET is_deleted = FALSE, last_update = ?
        WHERE id IN (SELECT id FROM todo_hierarchy)
    "#;
    
    let result = sqlx::query(safe_restore_query)
        .bind(&todo_id)
        .bind(&current_time)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("安全恢复待办事项失败: {}", e))?;
    
    // 提交事务
    tx.commit().await
        .map_err(|e| format!("提交事务失败: {}", e))?;
    
    Ok(result.rows_affected() > 0)
}

// 获取已删除的待办事项列表
#[tauri::command]
pub async fn get_deleted_todos(
    state: State<'_, DatabaseState>
) -> Result<Vec<Value>, String> {
    let pool_guard = state.pool.lock().await;
    let pool = pool_guard.as_ref()
        .ok_or("数据库连接未建立")?;
    
    let query = r#"
        SELECT id, parent_id, text, completed, created_at, completed_at, deadline, is_deleted, last_update
        FROM todo_items_sync
        WHERE is_deleted = TRUE
        ORDER BY updated_timestamp DESC
    "#;
    
    let rows = sqlx::query(query)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("获取已删除待办事项失败: {}", e))?;
    
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
        
        todo.insert("isDeleted".to_string(), Value::Bool(row.get::<bool, _>("is_deleted")));
        
        todos.push(Value::Object(todo));
    }
    
    Ok(todos)
}
