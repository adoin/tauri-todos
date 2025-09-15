// 数据库连接配置相关类型定义
import type { TodoItem } from './todo'
import type { AppSettings } from './app'

// 重新导出AppSettings类型
export type { AppSettings }

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

// 数据比较差异类型
export interface DataDifference {
  type: 'missing_local' | 'missing_remote' | 'content_diff' | 'no_diff'
  id?: string
  local: TodoItem | null
  remote: TodoItem | null
  title: string
  description: string
}

// 本地数据包
export interface LocalData {
  todos: TodoItem[]
  settings: AppSettings
  lastUpdate: string
}

// 远程数据包
export interface RemoteData {
  todos: TodoItem[]
  settings: AppSettings
  lastUpdate: string
}

// 连接状态类型
export type ConnectionStatus = 'checking' | 'connected' | 'failed' | 'no-config'

// 同步数据状态
export interface SyncDataState {
  localData: LocalData | null
  remoteData: RemoteData | null
  differences: DataDifference[]
  comparing: boolean
}

