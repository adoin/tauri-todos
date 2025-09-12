import { invoke } from '@tauri-apps/api/core'
import { computed, ref, watch } from 'vue'

// 应用状态接口
interface AppState {
  isTransparent: boolean
  showBorder: boolean
  isSettingsOpen: boolean
  activeToolbar: boolean
  windowConfig: {
    width: number
    height: number
    opacity: number
    borderRadius: number
    borderColor: string
    borderWidth: number
  }
  windowPosition: {
    x: number
    y: number
  }
}

// 默认状态
const defaultState: AppState = {
  isTransparent: false,
  showBorder: false,
  isSettingsOpen: false,
  activeToolbar: false,
  windowConfig: {
    width: 576,
    height: 756,
    opacity: 0.8,
    borderRadius: 8,
    borderColor: '#3b82f6',
    borderWidth: 2,
  },
  windowPosition: {
    x: 100,
    y: 100,
  },
}

// 响应式状态
const isTransparent = ref(defaultState.isTransparent)
const showBorder = ref(defaultState.showBorder)
const isSettingsOpen = ref(defaultState.isSettingsOpen)
const activeToolbar = ref(defaultState.activeToolbar)
const windowConfig = ref({ ...defaultState.windowConfig })
const windowPosition = ref({ ...defaultState.windowPosition })

// 计算属性
const windowStyle = computed(() => ({
  width: `${windowConfig.value.width}px`,
  height: `${windowConfig.value.height}px`,
  borderRadius: `${windowConfig.value.borderRadius}px`,
  border: showBorder.value ? `${windowConfig.value.borderWidth}px solid ${windowConfig.value.borderColor}` : 'none',
  opacity: windowConfig.value.opacity,
  backgroundColor: isTransparent.value ? 'transparent' : 'rgba(255, 255, 255, 0.9)',
}))

// 防抖保存
let saveTimeout: ReturnType<typeof setTimeout> | null = null
function debouncedSave() {
  if (saveTimeout)
    clearTimeout(saveTimeout)
  saveTimeout = setTimeout(async () => {
    await saveState()
    saveTimeout = null
  }, 1000)
}

// 保存状态到 JSON 文件
async function saveState() {
  try {
    const state: AppState = {
      isTransparent: isTransparent.value,
      showBorder: showBorder.value,
      isSettingsOpen: isSettingsOpen.value,
      activeToolbar: activeToolbar.value,
      windowConfig: windowConfig.value,
      windowPosition: windowPosition.value,
    }
    await invoke('save_app_state', { state })
    console.log('App state saved successfully')
  }
  catch (error) {
    console.error('Failed to save app state:', error)
  }
}

// 从 JSON 文件加载状态
async function loadState() {
  try {
    const state = await invoke('load_app_state') as AppState
    if (state) {
      isTransparent.value = state.isTransparent ?? defaultState.isTransparent
      showBorder.value = state.showBorder ?? defaultState.showBorder
      isSettingsOpen.value = false // 设置窗口总是关闭状态启动
      activeToolbar.value = state.activeToolbar ?? defaultState.activeToolbar

      if (state.windowConfig) {
        windowConfig.value = { ...defaultState.windowConfig, ...state.windowConfig }
      }

      if (state.windowPosition) {
        windowPosition.value = { ...state.windowPosition }
      }

      console.log('App state loaded successfully')
    }
  }
  catch (error) {
    console.error('Failed to load app state:', error)
  }
}

// 监听状态变化并自动保存
watch([isTransparent, activeToolbar, windowConfig, windowPosition], debouncedSave, { deep: true })

// Composable 函数
export function useAppState() {
  return {
    // 状态
    isTransparent,
    showBorder,
    isSettingsOpen,
    activeToolbar,
    windowConfig,
    windowPosition,

    // 计算属性
    windowStyle,

    // 方法
    toggleTransparency: () => {
      isTransparent.value = !isTransparent.value
    },

    toggleBorder: (show: boolean) => {
      showBorder.value = show
    },

    openSettings: () => {
      isSettingsOpen.value = true
    },

    closeSettings: () => {
      isSettingsOpen.value = false
    },

    updateWindowConfig: (config: Partial<typeof windowConfig.value>) => {
      windowConfig.value = { ...windowConfig.value, ...config }
    },

    updateWindowPosition: (position: { x: number, y: number }) => {
      windowPosition.value = { ...position }
    },

    updateActiveToolbar: (active: boolean) => {
      activeToolbar.value = active
    },

    // 持久化
    saveState,
    loadState,
  }
}
