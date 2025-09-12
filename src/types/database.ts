// 数据库连接配置相关类型定义

export interface DatabaseConfig {
  host: string
  port: number
  username: string
  password: string
  database: string
}

export interface SyncStatus {
  isConnected: boolean
  isSyncing: boolean
  lastSyncTime?: string
  error?: string
}

export interface SyncResult {
  success: boolean
  message: string
  data?: {
    localLastUpdate: string
    remoteLastUpdate: string
    syncedItems: number
  }
}

