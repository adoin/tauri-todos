import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { DatabaseConfig, SyncStatus, SyncResult } from '../types/database'

export const useSyncStore = defineStore('sync', () => {
  // 状态
  const syncStatus = ref<SyncStatus>({
    isConnected: false,
    isSyncing: false,
    lastSyncTime: undefined,
    error: undefined
  })

  const databaseConfig = ref<DatabaseConfig | null>(null)
  const autoSyncEnabled = ref(false)
  const syncDebounceTime = ref(2000) // 2秒防抖

  // 计算属性
  const isSyncAvailable = computed(() => {
    return syncStatus.value.isConnected && !syncStatus.value.isSyncing
  })

  const lastSyncDisplay = computed(() => {
    if (!syncStatus.value.lastSyncTime) return '从未同步'
    return new Date(syncStatus.value.lastSyncTime).toLocaleString('zh-CN')
  })

  // 方法
  const loadDatabaseConfig = async () => {
    try {
      const config = await invoke('load_database_config') as DatabaseConfig | null
      databaseConfig.value = config
      return config
    } catch (error) {
      console.error('加载数据库配置失败:', error)
      return null
    }
  }

  const testConnection = async (config: DatabaseConfig) => {
    try {
      syncStatus.value.error = undefined
      const result = await invoke('test_database_connection', { config })
      syncStatus.value.isConnected = result as boolean
      return result as boolean
    } catch (error) {
      syncStatus.value.isConnected = false
      syncStatus.value.error = error instanceof Error ? error.message : '连接失败'
      return false
    }
  }

  const connectDatabase = async (config: DatabaseConfig) => {
    try {
      await invoke('connect_database', { config })
      syncStatus.value.isConnected = true
      syncStatus.value.error = undefined
      databaseConfig.value = config
      return true
    } catch (error) {
      syncStatus.value.isConnected = false
      syncStatus.value.error = error instanceof Error ? error.message : '连接失败'
      return false
    }
  }

  const checkAndInitializeTables = async () => {
    try {
      const message = await invoke('check_and_initialize_tables') as string
      return message
    } catch (error) {
      throw new Error(error instanceof Error ? error.message : '表结构检查失败')
    }
  }

  const startSync = async (): Promise<SyncResult> => {
    if (!syncStatus.value.isConnected) {
      throw new Error('数据库未连接')
    }

    try {
      syncStatus.value.isSyncing = true
      syncStatus.value.error = undefined

      const result = await invoke('start_database_sync') as SyncResult

      if (result.success) {
        syncStatus.value.lastSyncTime = new Date().toISOString()
      }

      return result
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '同步失败'
      syncStatus.value.error = errorMessage
      throw new Error(errorMessage)
    } finally {
      syncStatus.value.isSyncing = false
    }
  }

  const enableAutoSync = () => {
    autoSyncEnabled.value = true
  }

  const disableAutoSync = () => {
    autoSyncEnabled.value = false
  }

  const setSyncDebounceTime = (time: number) => {
    syncDebounceTime.value = time
  }

  const clearSyncError = () => {
    syncStatus.value.error = undefined
  }

  const resetSyncStatus = () => {
    syncStatus.value = {
      isConnected: false,
      isSyncing: false,
      lastSyncTime: undefined,
      error: undefined
    }
    databaseConfig.value = null
    autoSyncEnabled.value = false
  }

  return {
    // 状态
    syncStatus,
    databaseConfig,
    autoSyncEnabled,
    syncDebounceTime,

    // 计算属性
    isSyncAvailable,
    lastSyncDisplay,

    // 方法
    loadDatabaseConfig,
    testConnection,
    connectDatabase,
    checkAndInitializeTables,
    startSync,
    enableAutoSync,
    disableAutoSync,
    setSyncDebounceTime,
    clearSyncError,
    resetSyncStatus
  }
})

