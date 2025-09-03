import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useAppStore = defineStore('app', () => {
  // 悬浮窗口状态
  const isTransparent = ref(true)
  const showBorder = ref(false)
  const isSettingsOpen = ref(false)

  // 窗口配置
  const windowConfig = ref({
    width: 400,
    height: 300,
    opacity: 0.8,
    borderRadius: 8,
    borderColor: '#3b82f6',
    borderWidth: 2
  })

  // 窗口位置
  const windowPosition = ref({
    x: 100,
    y: 100
  })

  // 计算属性
  const windowStyle = computed(() => ({
    width: `${windowConfig.value.width}px`,
    height: `${windowConfig.value.height}px`,
    borderRadius: `${windowConfig.value.borderRadius}px`,
    border: showBorder.value ? `${windowConfig.value.borderWidth}px solid ${windowConfig.value.borderColor}` : 'none',
    opacity: windowConfig.value.opacity,
    backgroundColor: isTransparent.value ? 'transparent' : 'rgba(255, 255, 255, 0.9)'
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

  const updateWindowPosition = (position: { x: number; y: number }) => {
    windowPosition.value = { ...position }
  }

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
    updateWindowPosition
  }
})
