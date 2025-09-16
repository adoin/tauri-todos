import type { LocaleKey } from '../constants/locale'
import type { AppSettings, WindowConfig } from '../types/app'

// 全局通知类型
export interface GlobalNotification {
  id: string
  type: 'info' | 'success' | 'warning' | 'error'
  message: string
  timestamp: string
  duration?: number // 显示时长（毫秒），0表示不自动消失
}
import { invoke } from '@tauri-apps/api/core'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { debounce } from 'xe-utils'
import { defaultLocale } from '../constants/locale'
import { defaultAppSettings } from '../constants/todo'
import { useSyncStore } from './sync'

export const useAppStore = defineStore('app', () => {
  const isSettingsOpen = ref(false)
  // 同步相关
  const syncStore = useSyncStore()
  // 待办事项设置
  const appSettings = ref<AppSettings>({ ...defaultAppSettings })
  
  // 全局通知状态
  const globalNotifications = ref<GlobalNotification[]>([])
  const currentNotification = ref<GlobalNotification | null>(null)

  // 计算属性
  const windowStyle = computed(() => ({
    borderRadius: `${appSettings.value.windowConfig.borderRadius}px`,
    border: appSettings.value.windowConfig.borderWidth > 0 ? `${appSettings.value.windowConfig.borderWidth}px solid ${appSettings.value.windowConfig.borderColor}` : 'none',
  }))

  const saveAppSettings = async () => {
    try {
      await invoke('save_app_settings', { settings: { ...appSettings.value, lastUpdate: new Date().toISOString() } })
      // 如果启用了自动同步，立即同步设置
      if (syncStore.autoSyncEnabled && syncStore.isSyncAvailable) {
        try {
          await syncStore.startSync()
          console.log('设置保存后自动同步完成')
        }
        catch (error) {
          console.error('设置保存后自动同步失败:', error)
          // 不抛出错误，避免影响设置保存
        }
      }
    }
    catch (err) {
      console.error('Failed to save todo settings:', err)
      throw err
    }
  }
  const debouncedSaveAppSettings = debounce(saveAppSettings, 1000)
  // 动作
  const toggleTransparency = () => {
    appSettings.value.isTransparent = !appSettings.value.isTransparent
    debouncedSaveAppSettings()
  }

  const toggleBorder = (show: boolean) => {
    appSettings.value.windowConfig.borderWidth = show ? 2 : 0
  }

  const openSettings = () => {
    isSettingsOpen.value = true
  }

  const closeSettings = () => {
    isSettingsOpen.value = false
  }
  const updateWindowConfig = (config: Partial<WindowConfig>) => {
    appSettings.value.windowConfig = { ...appSettings.value.windowConfig, ...config }
    debouncedSaveAppSettings()
  }

  const updateLocale = (newLocale?: LocaleKey) => {
    appSettings.value.locale = newLocale || defaultLocale
    debouncedSaveAppSettings()
  }

  const updateAppSettings = async (newSettings: Partial<AppSettings>) => {
    appSettings.value = { ...appSettings.value, ...newSettings }
    debouncedSaveAppSettings()
  }

  const resetColorsToDefault = async () => {
    appSettings.value.colors = { ...defaultAppSettings.colors }
    appSettings.value.windowConfig.borderColor = defaultAppSettings.windowConfig.borderColor
    debouncedSaveAppSettings()
  }

  const loadAppSettings = async () => {
    try {
      const settingsData = await invoke('load_app_settings') as AppSettings
      if (settingsData && settingsData.lastUpdate && (!appSettings.value.lastUpdate || new Date(settingsData.lastUpdate) > new Date(appSettings.value.lastUpdate))) {
        appSettings.value = settingsData
      }
      isSettingsOpen.value = false
    }
    catch (err) {
      console.error('Failed to load todo settings:', err)
      // 如果加载设置失败，使用默认设置
      appSettings.value = { ...defaultAppSettings }
    }
  }

  // 通知相关方法
  const showNotification = (type: GlobalNotification['type'], message: string, duration = 3000) => {
    const notification: GlobalNotification = {
      id: `notification_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      type,
      message,
      timestamp: new Date().toISOString(),
      duration,
    }

    // 添加到通知列表
    globalNotifications.value.push(notification)
    
    // 设置为当前通知
    currentNotification.value = notification

    // 如果设置了自动消失时间，则自动清除
    if (duration > 0) {
      setTimeout(() => {
        removeNotification(notification.id)
      }, duration)
    }
  }

  const removeNotification = (id: string) => {
    const index = globalNotifications.value.findIndex(n => n.id === id)
    if (index > -1) {
      globalNotifications.value.splice(index, 1)
    }
    
    // 如果移除的是当前通知，清除当前通知
    if (currentNotification.value?.id === id) {
      currentNotification.value = null
    }
  }

  const clearAllNotifications = () => {
    globalNotifications.value = []
    currentNotification.value = null
  }

  // 便捷方法
  const showInfo = (message: string, duration = 3000) => showNotification('info', message, duration)
  const showSuccess = (message: string, duration = 3000) => showNotification('success', message, duration)
  const showWarning = (message: string, duration = 5000) => showNotification('warning', message, duration)
  const showError = (message: string, duration = 0) => showNotification('error', message, duration) // 错误通知不自动消失

  return {
    // 状态
    isSettingsOpen,
    appSettings,
    windowStyle,
    globalNotifications,
    currentNotification,
    // 方法
    toggleTransparency,
    toggleBorder,
    openSettings,
    closeSettings,
    updateWindowConfig,
    updateLocale,
    updateAppSettings,
    resetColorsToDefault,
    saveAppSettings,
    loadAppSettings,
    // 通知方法
    showNotification,
    removeNotification,
    clearAllNotifications,
    showInfo,
    showSuccess,
    showWarning,
    showError,
  }
})
