<script setup lang="ts">
import type { GlobalNotification } from '../store/app'
import { computed } from 'vue'
import { useAppStore } from '../store/app'
import { useSyncStore } from '../store/sync'

const appStore = useAppStore()
const syncStore = useSyncStore()

// 计算属性
const hasNotification = computed(() => appStore.currentNotification !== null)
const notification = computed(() => appStore.currentNotification)

// 数据库连接状态
const dbConnectionStatus = computed(() => {
  switch (syncStore.connectionStatus) {
    case 'connected':
      return { text: '数据库已连接', color: 'bg-green-400', icon: '✓' }
    case 'checking':
      return { text: '检查连接中...', color: 'bg-yellow-400', icon: '⏳' }
    case 'failed':
      return { text: '数据库连接失败', color: 'bg-red-400', icon: '✗' }
    case 'no-config':
      return { text: '未配置数据库', color: 'bg-gray-400', icon: '○' }
    default:
      return { text: '未知状态', color: 'bg-gray-400', icon: '?' }
  }
})

// 自动同步状态
const autoSyncStatus = computed(() => {
  if (syncStore.isAutoSyncEnabled && syncStore.autoSyncInterval > 0) {
    return `自动同步: ${syncStore.formatAutoSyncInterval(syncStore.autoSyncInterval)}`
  }
  return '自动同步: 已禁用'
})

// 格式化时间显示
function formatTime(timestamp: string): string {
  const date = new Date(timestamp)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  
  if (diff < 60000) { // 1分钟内
    return '刚刚'
  }
  else if (diff < 3600000) { // 1小时内
    const minutes = Math.floor(diff / 60000)
    return `${minutes}分钟前`
  }
  else if (diff < 86400000) { // 24小时内
    const hours = Math.floor(diff / 3600000)
    return `${hours}小时前`
  }
  else {
    return date.toLocaleDateString('zh-CN')
  }
}

// 获取通知类型对应的样式
function getNotificationStyle(type: GlobalNotification['type']) {
  const styles = {
    info: 'bg-blue-50 text-blue-800 border-blue-200',
    success: 'bg-green-50 text-green-800 border-green-200',
    warning: 'bg-yellow-50 text-yellow-800 border-yellow-200',
    error: 'bg-red-50 text-red-800 border-red-200',
  }
  return styles[type]
}

// 获取通知类型对应的图标
function getNotificationIcon(type: GlobalNotification['type']) {
  const icons = {
    info: 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z',
    success: 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z',
    warning: 'M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z',
    error: 'M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z',
  }
  return icons[type]
}

// 关闭通知
function closeNotification() {
  if (notification.value) {
    appStore.removeNotification(notification.value.id)
  }
}
</script>

<template>
  <div class="global-footer">
    <!-- 通知区域 -->
    <div
      v-if="hasNotification"
      class="notification-bar"
      :class="getNotificationStyle(notification!.type)"
    >
      <div class="flex items-center justify-between w-full">
        <div class="flex items-center gap-2">
          <!-- 通知图标 -->
          <svg
            class="w-4 h-4 flex-shrink-0"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              :d="getNotificationIcon(notification!.type)"
            />
          </svg>
          
          <!-- 通知内容 -->
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium truncate">
              {{ notification!.message }}
            </p>
            <p class="text-xs opacity-75">
              {{ formatTime(notification!.timestamp) }}
            </p>
          </div>
        </div>
        
        <!-- 关闭按钮 -->
        <button
          class="flex-shrink-0 p-1 rounded-full hover:bg-black hover:bg-opacity-10 transition-colors"
          @click="closeNotification"
        >
          <svg
            class="w-3 h-3"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </button>
      </div>
    </div>
    
    <!-- 默认状态栏 -->
    <div
      v-else
      class="default-status-bar"
    >
      <div class="flex items-center justify-between w-full text-xs text-gray-500">
        <div class="flex items-center gap-4">
          <span>待办事项管理</span>
          <span>•</span>
          <span>{{ autoSyncStatus }}</span>
        </div>
        <div class="flex items-center gap-2">
          <div class="w-2 h-2 rounded-full" :class="dbConnectionStatus.color"></div>
          <span>{{ dbConnectionStatus.text }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.global-footer {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  z-index: 1000;
  border-top: 1px solid var(--todo-border-color);
  background-color: var(--window-background);
  backdrop-filter: blur(8px);
}

.notification-bar {
  padding: 8px 16px;
  border-bottom: 1px solid;
  animation: slideUp 0.3s ease-out;
}

.default-status-bar {
  padding: 6px 16px;
  background-color: rgba(255, 255, 255, 0.8);
}

@keyframes slideUp {
  from {
    transform: translateY(100%);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

/* 响应式设计 */
@media (max-width: 640px) {
  .global-footer {
    font-size: 12px;
  }
  
  .notification-bar,
  .default-status-bar {
    padding: 6px 12px;
  }
}
</style>

