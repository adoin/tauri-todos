# MySQL 同步功能数据库设计

## 表结构设计

> **注意**：此文档描述的是最新的数据库表结构，支持 UUID 格式的 ID 字段。

### 1. 设置同步表 (`todo_settings_sync`)

用于存储应用设置和同步元数据。

```sql
CREATE TABLE todo_settings_sync (
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
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
```

**字段说明：**
- `update_time`: 该设置项的更新时间
- `field_name`: 设置字段名（如：colors.normal, archiveDays 等）
- `data_type`: 数据类型（string, number, object, array）
- `field_value`: 字段值（JSON 字符串）
- `last_update`: 全局最后更新时间，用于同步判断

### 2. 待办同步表 (`todo_items_sync`)

用于存储待办事项数据，支持树形结构。

```sql
CREATE TABLE todo_items_sync (
    id VARCHAR(36) PRIMARY KEY COMMENT '待办事项ID (UUID)',
    parent_id VARCHAR(36) NULL COMMENT '父项ID，支持树形结构 (UUID)',
    text TEXT NOT NULL COMMENT '待办事项内容',
    completed BOOLEAN NOT NULL DEFAULT FALSE COMMENT '是否完成',
    created_at VARCHAR(50) NOT NULL COMMENT '创建时间',
    completed_at VARCHAR(50) NULL COMMENT '完成时间',
    deadline VARCHAR(50) NULL COMMENT '截止时间',
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE COMMENT '是否已删除（逻辑删除）',
    last_update VARCHAR(50) NOT NULL COMMENT '最后更新时间',
    created_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_id (id),
    INDEX idx_parent_id (parent_id),
    INDEX idx_completed (completed),
    INDEX idx_is_deleted (is_deleted),
    INDEX idx_last_update (last_update),
    INDEX idx_deadline (deadline),
    INDEX idx_created_timestamp (created_timestamp),
    INDEX idx_updated_timestamp (updated_timestamp)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
```

**字段说明：**
- `id`: 待办事项唯一标识符（UUID格式，36字符）
- `parent_id`: 父项ID，用于实现树形结构，NULL 表示根级项目（UUID格式，36字符）
- `text`: 待办事项文本内容
- `completed`: 完成状态
- `created_at`: 创建时间（ISO 字符串）
- `completed_at`: 完成时间（ISO 字符串）
- `deadline`: 截止时间（ISO 字符串）
- `is_deleted`: 逻辑删除标记，用于软删除功能
- `last_update`: 最后更新时间，用于同步判断

**索引说明：**
- `idx_id`: 主键索引，优化ID查询性能
- `idx_parent_id`: 父项ID索引，优化树形结构查询
- `idx_completed`: 完成状态索引，优化状态筛选
- `idx_is_deleted`: 删除状态索引，优化软删除查询
- `idx_last_update`: 更新时间索引，优化同步查询
- `idx_deadline`: 截止时间索引，优化时间相关查询
- `idx_created_timestamp`: 创建时间戳索引，优化排序查询
- `idx_updated_timestamp`: 更新时间戳索引，优化排序查询

## 同步策略

### 数据同步逻辑

1. **时间戳比较**: 使用 `last_update` 字段比较本地和远程数据的新旧
2. **双向同步**: 
   - 远程较新 → 覆盖本地
   - 本地较新 → 更新远程
   - 相同时间 → 跳过
3. **冲突处理**: 以时间戳为准，较新的数据优先

### 自动表管理

应用会自动：
- 检查表是否存在
- 不存在则创建表
- 存在但结构不匹配则自动修改表结构

## 安全考虑

- 数据库连接信息加密存储
- 使用参数化查询防止 SQL 注入
- 连接超时和重试机制
- 错误日志记录

