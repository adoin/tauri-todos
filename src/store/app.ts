import type { LocaleKey } from '../constants/locale'
import type { AppSettings, WindowConfig } from '../types/app'
import { invoke } from '@tauri-apps/api/core'
import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'
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

  return {
    // 状态
    isSettingsOpen,
    appSettings,
    windowStyle,
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
  }
})
