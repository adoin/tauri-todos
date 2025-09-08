<script setup lang="ts">
import type { PhysicalPosition, PhysicalSize } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { onMounted, onUnmounted, ref } from 'vue'
import { useAppStore } from '../store/app'
import TodoList from './TodoList.vue'

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
    class="relative backdrop-blur-10px transition-all duration-200 ease-in-out overflow-hidden rounded-lg w-screen h-screen bg-[#ffffff1a] border border-solid border-white/20"
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
  >
    <!-- 悬浮窗口内容 -->
    <div class="w-full h-full flex flex-col bg-white/10">
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
            title="设置"
            @click="openSettings"
          >
            ⚙️
          </button>
        </div>
      </div>

      <!-- 主要内容区域 -->
      <div class="flex-1 p-0 flex flex-col bg-white/5 rounded-b-lg overflow-hidden">
        <TodoList />
      </div>
    </div>
  </div>
</template>
