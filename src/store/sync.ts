import type { ConnectionStatus, DatabaseConfig, SyncResult, SyncStatus } from '../types/database'
import { invoke } from '@tauri-apps/api/core'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { useAppStore } from './app'
import { useTodoStore } from './todo'

export const useSyncStore = defineStore('sync', () => {
  // 状态
  const syncStatus = ref<SyncStatus>({
    isConnected: false,
    isSyncing: false,
    lastSyncTime: undefined,
    error: undefined,
  })

  const databaseConfig = ref<DatabaseConfig | null>(null)
  const connectionStatus = ref<ConnectionStatus>('checking')
  const syncDebounceTime = ref(2000) // 2秒防抖

  // 自动同步相关状态
  let autoSyncTimer: ReturnType<typeof window.setInterval> | null = null
  const autoSyncInterval = ref<number>(0) // 自动同步间隔（毫秒）
  const isAutoSyncEnabled = ref(false)
  const nextAutoSyncTime = ref<string>('')

  // 获取其他 store
  const appStore = useAppStore()
  const todoStore = useTodoStore()

  // 计算属性
  const isSyncAvailable = computed(() => {
    return syncStatus.value.isConnected && !syncStatus.value.isSyncing
  })

  const lastSyncDisplay = computed(() => {
    if (!syncStatus.value.lastSyncTime) {
      return '从未同步'
    }
    return new Date(syncStatus.value.lastSyncTime).toLocaleString('zh-CN')
  })

  // 方法
  const loadDatabaseConfig = async () => {
    try {
      const config = await invoke('load_database_config') as DatabaseConfig | null
      databaseConfig.value = config
      return config
    }
    catch (error) {
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
    }
    catch (error) {
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
    }
    catch (error) {
      syncStatus.value.isConnected = false
      syncStatus.value.error = error instanceof Error ? error.message : '连接失败'
      return false
    }
  }

  const checkAndInitializeTables = async () => {
    try {
      const message = await invoke('check_and_initialize_tables') as string
      return message
    }
    catch (error) {
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
    }
    catch (error) {
      const errorMessage = error instanceof Error ? error.message : '同步失败'
      syncStatus.value.error = errorMessage
      throw new Error(errorMessage)
    }
    finally {
      syncStatus.value.isSyncing = false
    }
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
      error: undefined,
    }
    databaseConfig.value = null
    connectionStatus.value = 'checking'
    stopAutoSync()
  }

  // 解析自动同步时间字符串
  function parseAutoSyncInterval(autoSyncStr: string): number {
    if (!autoSyncStr || autoSyncStr === '0') {
      return 0 // 不自动同步
    }

    const pattern = /^(\d+)([mh])$/
    const match = autoSyncStr.match(pattern)

    if (!match) {
      return 0
    }

    const num = Number.parseInt(match[1])
    const unit = match[2]

    if (unit === 'm') {
      return num * 60 * 1000 // 转换为毫秒
    }
    else if (unit === 'h') {
      return num * 60 * 60 * 1000 // 转换为毫秒
    }

    return 0
  }

  // 格式化自动同步间隔显示
  function formatAutoSyncInterval(interval: number): string {
    if (interval === 0) {
      return '未启用'
    }

    const minutes = Math.floor(interval / (60 * 1000))
    const hours = Math.floor(minutes / 60)

    if (hours > 0) {
      return `每 ${hours} 小时`
    }
    else {
      return `每 ${minutes} 分钟`
    }
  }

  // 启动自动同步
  function startAutoSync(interval: number) {
    if (interval <= 0) {
      return
    }

    // 清除现有计时器
    stopAutoSync()

    // 设置下次同步时间
    const nextTime = new Date(Date.now() + interval)
    nextAutoSyncTime.value = nextTime.toLocaleString('zh-CN')

    // 启动计时器
    autoSyncTimer = window.setInterval(async () => {
      if (connectionStatus.value === 'connected' && !syncStatus.value.isSyncing) {
        await performAutoSync()

        // 更新下次同步时间
        const nextTime = new Date(Date.now() + interval)
        nextAutoSyncTime.value = nextTime.toLocaleString('zh-CN')
      }
    }, interval)

    appStore.showInfo(`自动同步已启动，间隔: ${formatAutoSyncInterval(interval)}`)
  }

  // 停止自动同步
  function stopAutoSync() {
    if (autoSyncTimer !== null) {
      clearInterval(autoSyncTimer)
      autoSyncTimer = null
    }

    nextAutoSyncTime.value = ''
  }

  // 执行自动同步
  async function performAutoSync() {
    try {
      // 重新建立连接以确保连接有效
      const config = await loadDatabaseConfig()
      if (!config) {
        return
      }

      await connectDatabase(config)
      await checkAndInitializeTables()

      const result = await startSync()

      if (result.success) {
        appStore.showSuccess('自动同步成功')
        // 重新加载本地数据
        await todoStore.loadTodos()
      }
      else {
        appStore.showError(`自动同步失败: ${result.message}`)
      }
    }
    catch (error) {
      console.error('自动同步出错:', error)
    }
  }

  // 初始化自动同步
  async function initializeAutoSync() {
    try {
      const autoSync = appStore.appSettings.autoSync
      if (autoSync) {
        const interval = parseAutoSyncInterval(autoSync)
        if (interval > 0) {
          // 设置自动同步间隔，但不立即启动
          autoSyncInterval.value = interval
          isAutoSyncEnabled.value = true

          // 如果当前已连接，立即启动
          if (connectionStatus.value === 'connected') {
            startAutoSync(interval)
          }
        }
      }
    }
    catch (error) {
      console.error('初始化自动同步失败:', error)
    }
  }

  // 程序启动时自动连接数据库
  async function initializeDatabaseConnection() {
    try {
      connectionStatus.value = 'checking'

      // 1. 检查是否有数据库配置
      const config = await loadDatabaseConfig()
      if (!config) {
        connectionStatus.value = 'no-config'
        return
      }

      // 2. 测试数据库连接
      const isConnected = await testConnection(config)
      if (!isConnected) {
        connectionStatus.value = 'failed'
        return
      }

      // 3. 建立连接并检查表结构
      await connectDatabase(config)
      await checkAndInitializeTables()

      connectionStatus.value = 'connected'

      // 4. 初始化自动同步
      await initializeAutoSync()
    }
    catch (error) {
      console.error('数据库连接初始化失败:', error)
      connectionStatus.value = 'failed'
    }
  }

  // 处理连接状态变化（供 App.vue 调用）
  function handleConnectionStatusChange(newStatus: ConnectionStatus) {
    if (newStatus === 'connected' && isAutoSyncEnabled.value) {
      // 连接成功后，如果自动同步已启用，重新启动
      if (autoSyncInterval.value > 0) {
        startAutoSync(autoSyncInterval.value)
      }
    }
    else if (newStatus !== 'connected') {
      // 连接断开时停止自动同步计时器，但保留配置
      stopAutoSync()
    }
  }

  // 处理自动同步配置变化（供 App.vue 调用）
  function handleAutoSyncConfigChange(newAutoSync: string | undefined, oldAutoSync: string | undefined) {
    // 如果配置没有实际变化，跳过处理
    if (newAutoSync === oldAutoSync) {
      return
    }

    if (newAutoSync) {
      const regex = /^(\d+)([mh])$/
      const match = newAutoSync.match(regex)
      if (!match) {
        return
      }
      const interval = parseAutoSyncInterval(newAutoSync)
      if (interval > 0) {
        if (autoSyncTimer !== null) {
          clearInterval(autoSyncTimer)
          autoSyncTimer = null
        }

        nextAutoSyncTime.value = ''
        // 设置新的自动同步配置
        autoSyncInterval.value = interval
        isAutoSyncEnabled.value = true

        // 如果当前已连接，立即启动新的自动同步
        if (connectionStatus.value === 'connected') {
          startAutoSync(interval)
        }
      }
      else {
        // 设置为0，禁用自动同步
        autoSyncInterval.value = 0
        isAutoSyncEnabled.value = false
      }
    }
    else {
      // 配置为空，禁用自动同步
      autoSyncInterval.value = 0
      isAutoSyncEnabled.value = false
    }
  }

  return {
    // 状态
    syncStatus,
    databaseConfig,
    connectionStatus,
    syncDebounceTime,
    autoSyncInterval,
    isAutoSyncEnabled,
    nextAutoSyncTime,

    // 计算属性
    isSyncAvailable,
    lastSyncDisplay,

    // 方法
    loadDatabaseConfig,
    testConnection,
    connectDatabase,
    checkAndInitializeTables,
    startSync,
    setSyncDebounceTime,
    clearSyncError,
    resetSyncStatus,
    parseAutoSyncInterval,
    formatAutoSyncInterval,
    startAutoSync,
    stopAutoSync,
    performAutoSync,
    initializeAutoSync,
    initializeDatabaseConnection,
    handleConnectionStatusChange,
    handleAutoSyncConfigChange,
  }
})
