import { invoke } from '@tauri-apps/api/core'
import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'

export const useAppStore = defineStore('app', () => {
  // 悬浮窗口状态
  const isTransparent = ref(true)
  const showBorder = ref(false)
  const isSettingsOpen = ref(false)

  // 窗口配置
  const windowConfig = ref({
    width: 576, // 30% of 1920px
    height: 756, // 70% of 1080px
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

  // 计算属性
  const windowStyle = computed(() => ({
    width: `${windowConfig.value.width}px`,
    height: `${windowConfig.value.height}px`,
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

  // 保存状态到本地 JSON 文件
  const saveState = async () => {
    try {
      const state = {
        isTransparent: isTransparent.value,
        showBorder: showBorder.value,
        isSettingsOpen: isSettingsOpen.value,
        windowConfig: windowConfig.value,
        windowPosition: windowPosition.value,
      }
      await invoke('save_app_state', { state })
    }
    catch (error) {
      console.error('Failed to save app state:', error)
    }
  }

  // 从本地 JSON 文件加载状态
  const loadState = async () => {
    try {
      const state = await invoke('load_app_state') as any
      if (state) {
        isTransparent.value = state.isTransparent ?? true
        showBorder.value = state.showBorder ?? false
        isSettingsOpen.value = false // 设置窗口总是关闭状态启动
        if (state.windowConfig) {
          windowConfig.value = { ...windowConfig.value, ...state.windowConfig }
        }
        if (state.windowPosition) {
          windowPosition.value = { ...state.windowPosition }
        }
      }
    }
    catch (error) {
      console.error('Failed to load app state:', error)
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
  watch([isTransparent, windowConfig, windowPosition], debouncedSave, { deep: true })

  return {
    // 状态
    isTransparent,
    showBorder,
    isSettingsOpen,
    windowConfig,
    windowPosition,
    // 计算属性
    windowStyle,

    // 动作
    toggleTransparency,
    toggleBorder,
    openSettings,
    closeSettings,
    updateWindowConfig,
    updateWindowPosition,

    // 持久化相关
    saveState,
    loadState,
  }
})
