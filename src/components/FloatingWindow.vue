<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useAppStore } from '../store/app'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'

const appStore = useAppStore()
const isDragging = ref(false)
const dragOffset = ref({ x: 0, y: 0 })
const windowElement = ref<HTMLElement>()

const handleMouseEnter = () => {
  appStore.toggleBorder(true)
}

const handleMouseLeave = () => {
  appStore.toggleBorder(false)
}

const handleMouseDown = async (event: MouseEvent) => {
  if (event.target !== windowElement.value) return

  isDragging.value = true
  const window = getCurrentWindow()

  // 获取鼠标相对于窗口的位置
  const rect = windowElement.value!.getBoundingClientRect()
  dragOffset.value = {
    x: event.clientX - rect.left,
    y: event.clientY - rect.top
  }

  // 开始拖拽
  await window.startDragging()
}

let saveTimeout: NodeJS.Timeout | null = null

const handleMouseUp = () => {
  isDragging.value = false

  // 如果正在拖拽，延迟保存位置以避免过于频繁的保存
  if (saveTimeout) {
    clearTimeout(saveTimeout)
  }

  saveTimeout = setTimeout(() => {
    saveWindowConfig()
    saveTimeout = null
  }, 500)
}

const openSettings = () => {
  appStore.openSettings()
}

const saveWindowConfig = async () => {
  try {
    const window = getCurrentWindow()
    const position = await window.outerPosition()
    const size = await window.outerSize()

    await invoke('save_window_config', {
      config: {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height
      }
    })

    // 更新 store 中的位置
    appStore.updateWindowPosition({ x: position.x, y: position.y })
  } catch (error) {
    console.error('Failed to save window config:', error)
  }
}

const loadWindowConfig = async () => {
  try {
    const config = await invoke('load_window_config') as any
    appStore.updateWindowPosition({ x: config.x, y: config.y })

    // 设置窗口位置
    const window = getCurrentWindow()
    await window.setPosition({ x: config.x, y: config.y })
  } catch (error) {
    console.error('Failed to load window config:', error)
  }
}

// 监听鼠标事件
onMounted(async () => {
  document.addEventListener('mouseup', handleMouseUp)

  // 加载窗口配置
  await loadWindowConfig()
})

onUnmounted(() => {
  document.removeEventListener('mouseup', handleMouseUp)

  // 清除定时器
  if (saveTimeout) {
    clearTimeout(saveTimeout)
    saveTimeout = null
  }
})
</script>

<template>
  <div
    ref="windowElement"
    :style="appStore.windowStyle"
    class="floating-window"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
    @mousedown="handleMouseDown"
  >
    <!-- 悬浮窗口内容 -->
    <div class="floating-content">
      <!-- 顶部工具栏 -->
      <div class="toolbar">
        <div class="drag-handle">
          <span class="app-title">Ton</span>
        </div>
        <div class="toolbar-buttons">
          <button
            class="toolbar-btn settings-btn"
            @click="openSettings"
            title="设置"
          >
            ⚙️
          </button>
          <button
            class="toolbar-btn close-btn"
            @click="() => getCurrentWindow().close()"
            title="关闭"
          >
            ✕
          </button>
        </div>
      </div>

      <!-- 主要内容区域 -->
      <div class="main-area">
        <div class="welcome-message">
          <h3 class="text-lg font-semibold text-gray-800 mb-2">欢迎使用 Ton</h3>
          <p class="text-sm text-gray-600">这是一个桌面悬浮工具</p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.floating-window {
  position: relative;
  backdrop-filter: blur(10px);
  transition: all 0.2s ease;
  overflow: hidden;
}

.floating-content {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: rgba(255, 255, 255, 0.1);
}

.toolbar {
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  background: rgba(255, 255, 255, 0.2);
  backdrop-filter: blur(5px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.2);
}

.drag-handle {
  flex: 1;
  cursor: move;
  user-select: none;
}

.app-title {
  font-size: 16px;
  font-weight: 600;
  color: #1f2937;
  opacity: 0.8;
}

.toolbar-buttons {
  display: flex;
  gap: 8px;
}

.toolbar-btn {
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.3);
  color: #374151;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  transition: all 0.2s ease;
  backdrop-filter: blur(5px);
}

.toolbar-btn:hover {
  background: rgba(255, 255, 255, 0.5);
  transform: scale(1.05);
}

.close-btn:hover {
  background: rgba(239, 68, 68, 0.8);
  color: white;
}

.main-area {
  flex: 1;
  padding: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.05);
}

.welcome-message {
  text-align: center;
  color: rgba(31, 41, 55, 0.8);
}

.welcome-message h3 {
  margin: 0 0 8px 0;
  font-size: 18px;
  font-weight: 600;
}

.welcome-message p {
  margin: 0;
  font-size: 14px;
  opacity: 0.7;
}
</style>
