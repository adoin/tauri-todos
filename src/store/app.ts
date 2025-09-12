import type { LocaleKey } from '../constants/locale'
import { invoke } from '@tauri-apps/api/core'
import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'
import { defaultLocale } from '../constants/locale'

export const useAppStore = defineStore('app', () => {
  // 悬浮窗口状态
  const isTransparent = ref(true)
  const showBorder = ref(false)
  const isSettingsOpen = ref(false)

  // 窗口配置（不包含宽高，宽高由Tauri窗口管理）
  const windowConfig = ref({
    opacity: 0.8,
    borderRadius: 8,
    borderColor: '#3b82f6',
    borderWidth: 2,
  })

  // 窗口位置
  const windowPosition = ref({
    x: 100,
    y: 100,
  })

  // 语言配置
  const locale = ref<LocaleKey>(defaultLocale)

  // 计算属性
  const windowStyle = computed(() => ({
    borderRadius: `${windowConfig.value.borderRadius}px`,
    border: showBorder.value ? `${windowConfig.value.borderWidth}px solid ${windowConfig.value.borderColor}` : 'none',
    opacity: windowConfig.value.opacity,
    backgroundColor: isTransparent.value ? 'transparent' : 'rgba(255, 255, 255, 0.9)',
  }))

  // 动作
  const toggleTransparency = () => {
    isTransparent.value = !isTransparent.value
  }

  const toggleBorder = (show: boolean) => {
    showBorder.value = show
  }

  const openSettings = () => {
    isSettingsOpen.value = true
  }

  const closeSettings = () => {
    isSettingsOpen.value = false
  }

  const updateWindowConfig = (config: Partial<typeof windowConfig.value>) => {
    windowConfig.value = { ...windowConfig.value, ...config }
  }

  const updateWindowPosition = (position: { x: number, y: number }) => {
    windowPosition.value = { ...position }
  }

  const updateLocale = (newLocale: LocaleKey) => {
    locale.value = newLocale
  }

  // 保存应用设置到本地 JSON 文件
  const saveState = async () => {
    try {
      const settings = {
        isTransparent: isTransparent.value,
        showBorder: showBorder.value,
        isSettingsOpen: isSettingsOpen.value,
        windowConfig: windowConfig.value,
        locale: locale.value,
      }
      await invoke('save_app_settings', { settings })
    }
    catch (error) {
      console.error('Failed to save app settings:', error)
    }
  }

  // 从本地 JSON 文件加载应用设置
  const loadState = async () => {
    try {
      const settings = await invoke('load_app_settings') as any
      if (settings) {
        isTransparent.value = settings.isTransparent ?? true
        showBorder.value = settings.showBorder ?? false
        isSettingsOpen.value = false // 设置窗口总是关闭状态启动
        if (settings.windowConfig) {
          windowConfig.value = { ...windowConfig.value, ...settings.windowConfig }
        }
        if (settings.locale) {
          locale.value = settings.locale
        }
      }
    }
    catch (error) {
      console.error('Failed to load app settings:', error)
    }
  }

  // 监听状态变化并自动保存（防抖）
  let saveTimeout: ReturnType<typeof setTimeout> | null = null
  const debouncedSave = () => {
    if (saveTimeout)
      clearTimeout(saveTimeout)
    saveTimeout = setTimeout(() => {
      saveState()
      saveTimeout = null
    }, 1000) // 1秒防抖
  }

  // 监听需要持久化的状态变化
  watch([isTransparent, windowConfig, locale], debouncedSave, { deep: true })

  return {
    // 状态
    isTransparent,
    showBorder,
    isSettingsOpen,
    windowConfig,
    windowPosition,
    locale,
    // 计算属性
    windowStyle,

    // 动作
    toggleTransparency,
    toggleBorder,
    openSettings,
    closeSettings,
    updateWindowConfig,
    updateWindowPosition,
    updateLocale,

    // 持久化相关
    saveState,
    loadState,
  }
})
