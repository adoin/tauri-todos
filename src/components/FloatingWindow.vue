<script setup lang="ts">
import type { PhysicalPosition, PhysicalSize } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { onMounted, onUnmounted, ref } from 'vue'
import { useAppStore } from '../store/app'
import { useTodoStore } from '../store/todo'
import SyncModal from './SyncModal.vue'
import TodoList from './TodoList.vue'

const appStore = useAppStore()
const todoStore = useTodoStore()
const isDragging = ref(false)
// const dragOffset = ref({ x: 0, y: 0 })
const windowElement = ref<HTMLElement>()
const showToolbarItems = ref(false)
const syncModalRef = ref<InstanceType<typeof SyncModal>>()
function handleMouseEnter() {
  appStore.toggleBorder(true)
}

function handleMouseLeave() {
  appStore.toggleBorder(false)
}

async function handleMouseDown(_event: MouseEvent) {
  // 只有在拖拽手柄上才处理拖拽，不阻止其他事件
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

function exportData() {
  todoStore.exportTodos()
}

function importData() {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = '.json'
  input.onchange = (event) => {
    const file = (event.target as HTMLInputElement).files?.[0]
    if (file) {
      todoStore.importTodos(file)
    }
  }
  input.click()
}
function showSyncModal() {
  syncModalRef.value?.open()
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

    // 窗口尺寸由Tauri直接管理，不需要更新store
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

  // 加载应用状态（包含窗口配置）
  await appStore.loadState()

  // 加载窗口配置（位置和尺寸）
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
    class="relative backdrop-blur-10px transition-all duration-200 ease-in-out overflow-hidden floating-window"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
  >
    <!-- 悬浮窗口内容 -->
    <div class="w-full h-full flex flex-col">
      <!-- 顶部工具栏 -->
      <div
        class="h-10 flex items-center justify-between px-3 transition-all duration-200 ease-in-out hover:bg-white/20 hover:backdrop-blur-5px hover:border-b hover:border-white/20 hover:rounded-t-lg"
        @mouseenter="showToolbarItems = true"
        @mouseleave="showToolbarItems = false"
      >
        <div class="flex-1 cursor-move select-none" @mousedown="handleMouseDown">
          <span
            class="text-base font-semibold text-gray-800 transition-opacity duration-300 ease-in-out"
            :class="showToolbarItems ? 'opacity-80' : 'opacity-0'"
          >
            Ton
          </span>
        </div>
        <div
          class="flex gap-2 transition-opacity duration-300 ease-in-out"
          :class="showToolbarItems ? 'opacity-100' : 'opacity-0'"
        >
          <button
            class="w-7 h-7 border-none rounded-md bg-white/30 text-gray-700 cursor-pointer flex items-center justify-center text-sm transition-all duration-200 ease-in-out backdrop-blur-5px hover:bg-white/50 hover:scale-105"
            title="导出数据"
            @click="exportData"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24"><path fill="currentColor" d="M8.71 7.71L11 5.41V15a1 1 0 0 0 2 0V5.41l2.29 2.3a1 1 0 0 0 1.42 0a1 1 0 0 0 0-1.42l-4-4a1 1 0 0 0-.33-.21a1 1 0 0 0-.76 0a1 1 0 0 0-.33.21l-4 4a1 1 0 1 0 1.42 1.42M21 14a1 1 0 0 0-1 1v4a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1v-4a1 1 0 0 0-2 0v4a3 3 0 0 0 3 3h14a3 3 0 0 0 3-3v-4a1 1 0 0 0-1-1" /></svg>
          </button>
          <button
            class="w-7 h-7 border-none rounded-md bg-white/30 text-gray-700 cursor-pointer flex items-center justify-center text-sm transition-all duration-200 ease-in-out backdrop-blur-5px hover:bg-white/50 hover:scale-105"
            title="导入数据"
            @click="importData"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24"><path fill="currentColor" d="M21 14a1 1 0 0 0-1 1v4a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1v-4a1 1 0 0 0-2 0v4a3 3 0 0 0 3 3h14a3 3 0 0 0 3-3v-4a1 1 0 0 0-1-1m-9.71 1.71a1 1 0 0 0 .33.21a.94.94 0 0 0 .76 0a1 1 0 0 0 .33-.21l4-4a1 1 0 0 0-1.42-1.42L13 12.59V3a1 1 0 0 0-2 0v9.59l-2.29-2.3a1 1 0 1 0-1.42 1.42Z" /></svg>
          </button>
          <button
            class="w-7 h-7 border-none rounded-md bg-white/30 text-gray-700 cursor-pointer flex items-center justify-center text-sm transition-all duration-200 ease-in-out backdrop-blur-5px hover:bg-white/50 hover:scale-105"
            title="同步"
            @click="showSyncModal"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24"><path fill="currentColor" d="M13.03 18c.05.7.21 1.38.47 2h-7c-1.5 0-2.81-.5-3.89-1.57C1.54 17.38 1 16.09 1 14.58q0-1.95 1.17-3.48C3.34 9.57 4 9.43 5.25 9.15c.42-1.53 1.25-2.77 2.5-3.72S10.42 4 12 4c1.95 0 3.6.68 4.96 2.04S19 9.05 19 11h.1c-.74.07-1.45.23-2.1.5V11c0-1.38-.5-2.56-1.46-3.54C14.56 6.5 13.38 6 12 6s-2.56.5-3.54 1.46C7.5 8.44 7 9.62 7 11h-.5c-.97 0-1.79.34-2.47 1.03c-.69.68-1.03 1.5-1.03 2.47s.34 1.79 1.03 2.5c.68.66 1.5 1 2.47 1zM19 13.5V12l-2.25 2.25L19 16.5V15a2.5 2.5 0 0 1 2.5 2.5c0 .4-.09.78-.26 1.12l1.09 1.09c.42-.63.67-1.39.67-2.21c0-2.21-1.79-4-4-4m0 6.5a2.5 2.5 0 0 1-2.5-2.5c0-.4.09-.78.26-1.12l-1.09-1.09c-.42.63-.67 1.39-.67 2.21c0 2.21 1.79 4 4 4V23l2.25-2.25L19 18.5z" /></svg>
          </button>
          <button
            class="w-7 h-7 border-none rounded-md bg-white/30 text-gray-700 cursor-pointer flex items-center justify-center text-sm transition-all duration-200 ease-in-out backdrop-blur-5px hover:bg-white/50 hover:scale-105"
            title="设置"
            @click="openSettings"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24"><path fill="currentColor" fill-rule="evenodd" d="M12.563 3.2h-1.126l-.645 2.578l-.647.2a6.3 6.3 0 0 0-1.091.452l-.599.317l-2.28-1.368l-.796.797l1.368 2.28l-.317.598a6.3 6.3 0 0 0-.453 1.091l-.199.647l-2.578.645v1.126l2.578.645l.2.647q.173.568.452 1.091l.317.599l-1.368 2.28l.797.796l2.28-1.368l.598.317q.523.278 1.091.453l.647.199l.645 2.578h1.126l.645-2.578l.647-.2a6.3 6.3 0 0 0 1.091-.452l.599-.317l2.28 1.368l.796-.797l-1.368-2.28l.317-.598q.278-.523.453-1.091l.199-.647l2.578-.645v-1.126l-2.578-.645l-.2-.647a6.3 6.3 0 0 0-.452-1.091l-.317-.599l1.368-2.28l-.797-.796l-2.28 1.368l-.598-.317a6.3 6.3 0 0 0-1.091-.453l-.647-.199zm2.945 2.17l1.833-1.1a1 1 0 0 1 1.221.15l1.018 1.018a1 1 0 0 1 .15 1.221l-1.1 1.833q.33.62.54 1.3l2.073.519a1 1 0 0 1 .757.97v1.438a1 1 0 0 1-.757.97l-2.073.519q-.21.68-.54 1.3l1.1 1.833a1 1 0 0 1-.15 1.221l-1.018 1.018a1 1 0 0 1-1.221.15l-1.833-1.1q-.62.33-1.3.54l-.519 2.073a1 1 0 0 1-.97.757h-1.438a1 1 0 0 1-.97-.757l-.519-2.073a7.5 7.5 0 0 1-1.3-.54l-1.833 1.1a1 1 0 0 1-1.221-.15L4.42 18.562a1 1 0 0 1-.15-1.221l1.1-1.833a7.5 7.5 0 0 1-.54-1.3l-2.073-.519A1 1 0 0 1 2 12.72v-1.438a1 1 0 0 1 .757-.97l2.073-.519q.21-.68.54-1.3L4.27 6.66a1 1 0 0 1 .15-1.221L5.438 4.42a1 1 0 0 1 1.221-.15l1.833 1.1q.62-.33 1.3-.54l.519-2.073A1 1 0 0 1 11.28 2h1.438a1 1 0 0 1 .97.757l.519 2.073q.68.21 1.3.54zM12 14.8a2.8 2.8 0 1 0 0-5.6a2.8 2.8 0 0 0 0 5.6m0 1.2a4 4 0 1 1 0-8a4 4 0 0 1 0 8" /></svg>
          </button>
        </div>
      </div>

      <!-- 主要内容区域 -->
      <div class="flex-1 p-0 flex flex-col bg-white/5 rounded-b-lg overflow-hidden">
        <TodoList />
      </div>
    </div>

    <!-- 同步模态框 -->
    <SyncModal ref="syncModalRef" />
  </div>
</template>

<style scoped>
.floating-window {
  width: 100%;
  height: 100%;
  border-radius: var(--window-border-radius, 8px);
  border: var(--window-border-width, 2px) solid var(--window-border-color, #3b82f6);
  background-color: var(--window-background, transparent);
  opacity: var(--window-opacity, 0.8);
}
</style>
