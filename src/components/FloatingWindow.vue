<script setup lang="ts">
import type { PhysicalPosition, PhysicalSize } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { onMounted, onUnmounted, ref } from 'vue'
import { useAppStore } from '../store/app'

const appStore = useAppStore()
const isDragging = ref(false)
// const dragOffset = ref({ x: 0, y: 0 })
const windowElement = ref<HTMLElement>()
const showToolbarItems = ref(false)

function handleMouseEnter() {
  appStore.toggleBorder(true)
}

function handleMouseLeave() {
  appStore.toggleBorder(false)
}

async function handleMouseDown(event: MouseEvent) {
  // 检查是否点击在按钮上，如果是则不开始拖拽
  const button = (event.target as HTMLElement)?.closest('button')
  if (button)
    return

  isDragging.value = true
  const window = getCurrentWindow()

  // 开始拖拽
  await window.startDragging()
}

let saveTimeout: ReturnType<typeof setTimeout> | null = null

function handleMouseUp() {
  const wasDragging = isDragging.value
  isDragging.value = false

  // 如果正在拖拽，延迟保存位置以避免过于频繁的保存
  if (wasDragging) {
    if (saveTimeout) {
      clearTimeout(saveTimeout)
    }

    saveTimeout = setTimeout(() => {
      saveWindowConfig()
      saveTimeout = null
    }, 500)
  }
}

function openSettings() {
  appStore.openSettings()
}

async function saveWindowConfig() {
  try {
    const window = getCurrentWindow()
    const position = await window.outerPosition()
    const size = await window.outerSize()

    await invoke('save_window_config', {
      config: {
        x: position.x,
        y: position.y,
        width: size.width,
        height: size.height,
      },
    })

    // 更新 store 中的位置
    appStore.updateWindowPosition({ x: position.x, y: position.y })
  }
  catch (error) {
    console.error('Failed to save window config:', error)
  }
}

async function loadWindowConfig() {
  try {
    const config = await invoke('load_window_config') as any
    appStore.updateWindowPosition({ x: config.x, y: config.y })

    // 设置窗口位置和尺寸
    const window = getCurrentWindow()

    // 使用 Physical 位置格式
    await window.setPosition({
      type: 'Physical',
      x: Math.round(config.x),
      y: Math.round(config.y),
    } as PhysicalPosition)

    // 使用 Physical 尺寸格式
    await window.setSize({
      type: 'Physical',
      width: Math.round(config.width),
      height: Math.round(config.height),
    } as PhysicalSize)

    // 更新 store 中的窗口配置
    appStore.updateWindowConfig({
      width: config.width,
      height: config.height,
    })
  }
  catch (error) {
    console.error('Failed to load window config:', error)
  }
}

// 监听窗口尺寸变化
function handleWindowResize() {
  // 延迟保存以避免频繁保存
  if (saveTimeout) {
    clearTimeout(saveTimeout)
  }

  saveTimeout = setTimeout(() => {
    saveWindowConfig()
    saveTimeout = null
  }, 1000) // 窗口调整时延迟更长时间
}

// 监听鼠标事件
onMounted(async () => {
  document.addEventListener('mouseup', handleMouseUp)

  // 监听窗口尺寸变化
  const window = getCurrentWindow()
  window.listen('tauri://resize', handleWindowResize)

  // 监听托盘设置事件
  window.listen('open-settings', () => {
    appStore.openSettings()
  })

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
    class="floating-window"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
    @mousedown="handleMouseDown"
  >
    <!-- 悬浮窗口内容 -->
    <div class="floating-content">
      <!-- 顶部工具栏 -->
      <div class="toolbar" @mouseenter="showToolbarItems = true" @mouseleave="showToolbarItems = false">
        <div class="drag-handle">
          <span class="app-title" :class="{ visible: showToolbarItems }">Ton</span>
        </div>
        <div class="toolbar-buttons" :class="{ visible: showToolbarItems }">
          <button
            class="toolbar-btn settings-btn"
            title="设置"
            @click="openSettings"
          >
            ⚙️
          </button>
        </div>
      </div>

      <!-- 主要内容区域 -->
      <div class="main-area">
        <div class="welcome-message">
          <h3 class="text-lg font-semibold text-gray-800 mb-2">
            欢迎使用 Ton
          </h3>
          <p class="text-sm text-gray-600">
            这是一个桌面悬浮工具
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.floating-window {
  position: relative;
  backdrop-filter: blur(10px);
  transition: all 0.2s ease;
  overflow: hidden;
  border-radius: 8px;
  width: 100vw;
  height: 100vh;
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
  &:hover {
    background: rgba(255, 255, 255, 0.2);
    backdrop-filter: blur(5px);
    border-bottom: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 8px 8px 0 0;
  }
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
  opacity: 0;
  transition: opacity 0.3s ease;
}

.app-title.visible {
  opacity: 0.8;
}

.toolbar-buttons {
  display: flex;
  gap: 8px;
  opacity: 0;
  transition: opacity 0.3s ease;
}

.toolbar-buttons.visible {
  opacity: 1;
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

.main-area {
  flex: 1;
  padding: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 0 0 8px 8px;
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
