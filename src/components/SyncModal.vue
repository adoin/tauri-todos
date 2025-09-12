<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { ElButton, ElDialog, ElLoading, ElMessage } from 'element-plus'
import { computed, ref } from 'vue'
import { useAppStore } from '../store/app'
import { useTodoStore } from '../store/todo'

const appStore = useAppStore()
const todoStore = useTodoStore()

// 模态框状态
const visible = ref(false)
const loading = ref(false)
const syncLoading = ref(false)

// 数据状态
const hasConfig = ref(false)
const connectionStatus = ref<'checking' | 'connected' | 'failed' | 'no-config'>('checking')
const syncData = ref<any>(null)
const differences = ref<any[]>([])

// 计算属性
const hasDifferences = computed(() => differences.value.length > 0)

// 打开模态框
async function open() {
  visible.value = true
  await checkDatabaseStatus()
}

// 关闭模态框
function close() {
  visible.value = false
  resetState()
}

// 重置状态
function resetState() {
  connectionStatus.value = 'checking'
  syncData.value = null
  differences.value = []
  loading.value = false
  syncLoading.value = false
}

// 检查数据库状态
async function checkDatabaseStatus() {
  loading.value = true
  connectionStatus.value = 'checking'

  try {
    // 1. 检查是否有数据库配置
    const config = await invoke('load_database_config') as any
    if (!config) {
      connectionStatus.value = 'no-config'
      hasConfig.value = false
      return
    }

    hasConfig.value = true

    // 2. 测试数据库连接
    const isConnected = await invoke('test_database_connection', { config })
    if (!isConnected) {
      connectionStatus.value = 'failed'
      return
    }

    connectionStatus.value = 'connected'

    // 3. 建立连接并检查表结构
    await invoke('connect_database', { config })
    await invoke('check_and_initialize_tables')

    // 4. 比较数据
    await compareData()
    
    console.log('数据库连接检查完成，状态:', connectionStatus.value)
  }
  catch (error) {
    console.error('Database check failed:', error)
    connectionStatus.value = 'failed'
    ElMessage.error('数据库连接失败，请检查配置')
  }
  finally {
    loading.value = false
  }
}

// 比较本地和远程数据
async function compareData() {
  try {
    // 获取本地数据
    const localTodos = await invoke('load_todos') as any
    const localSettings = await invoke('load_settings') as any

    // 获取远程数据 - 这些函数是内部的，需要通过其他方式获取
    // 暂时跳过远程数据比较，直接显示连接成功
    const remoteTodos: any[] = []
    const remoteSettings: any = {}

    // 比较数据差异
    const diffs = compareDataDifferences(localTodos, remoteTodos, localSettings, remoteSettings)
    differences.value = diffs
  }
  catch (error) {
    console.error('Data comparison failed:', error)
    ElMessage.error('数据比较失败')
  }
}

// 比较数据差异
function compareDataDifferences(localTodos: any, _remoteTodos: any, _localSettings: any, _remoteSettings: any) {
  const diffs: any[] = []

  // 由于我们暂时无法获取真实的远程数据，这里模拟一些差异检测
  // 实际应用中，这里应该调用后端API来获取远程数据进行比较

  // 检查本地是否有数据
  const localTodoCount = localTodos.data?.length || 0
  const localHasData = localTodoCount > 0

  if (localHasData) {
    // 模拟检测到差异（实际应该与远程数据比较）
    diffs.push({
      type: 'data_exists',
      title: '检测到本地数据',
      local: localTodoCount,
      remote: '未知',
      description: `本地有 ${localTodoCount} 个待办事项，建议进行同步以确保数据一致性`,
    })
  }
  else {
    diffs.push({
      type: 'no_local_data',
      title: '本地无数据',
      local: 0,
      remote: '未知',
      description: '本地暂无待办事项数据，建议从远程拉取数据',
    })
  }

  return diffs
}

// 执行同步
async function performSync() {
  syncLoading.value = true

  try {
    // 确保数据库连接有效
    if (connectionStatus.value !== 'connected') {
      ElMessage.error('数据库连接未建立，请重新检查连接')
      return
    }

    // 重新建立连接以确保连接有效
    const config = await invoke('load_database_config') as any
    if (!config) {
      ElMessage.error('数据库配置不存在')
      return
    }

    await invoke('connect_database', { config })
    await invoke('check_and_initialize_tables')

    const result = await invoke('start_database_sync') as any

    if (result.success) {
      ElMessage.success(result.message)

      // 重新加载本地数据
      await todoStore.loadTodos()

      // 重新比较数据
      await compareData()
    }
    else {
      ElMessage.error(result.message || '同步失败')
    }
  }
  catch (error) {
    console.error('Sync failed:', error)
    const errorMessage = error instanceof Error ? error.message : String(error)
    ElMessage.error(`同步失败: ${errorMessage}`)
  }
  finally {
    syncLoading.value = false
  }
}

// 强制推送本地数据
async function forcePush() {
  syncLoading.value = true

  try {
    // 确保数据库连接有效
    if (connectionStatus.value !== 'connected') {
      ElMessage.error('数据库连接未建立，请重新检查连接')
      return
    }

    // 重新建立连接以确保连接有效
    const config = await invoke('load_database_config') as any
    if (!config) {
      ElMessage.error('数据库配置不存在')
      return
    }

    await invoke('connect_database', { config })
    await invoke('check_and_initialize_tables')

    // 使用同步功能，但强制推送本地数据
    // 这里需要修改本地时间戳来确保本地数据被认为是更新的
    const localTodos = await invoke('load_todos') as any

    // 更新本地时间戳为未来时间，确保本地数据被认为是更新的
    const futureTime = new Date(Date.now() + 60000).toISOString() // 1分钟后
    const updatedTodos = {
      ...localTodos,
      lastUpdate: futureTime,
    }

    // 保存更新后的时间戳
    await invoke('save_todos', { todos: updatedTodos })

    // 执行同步，由于本地时间更新，会自动推送本地数据
    const syncResult = await invoke('start_database_sync') as any
    if (!syncResult.success) {
      throw new Error(syncResult.message || '同步失败')
    }

    ElMessage.success('强制推送成功')

    // 重新比较数据
    await compareData()
  }
  catch (error) {
    console.error('Force push failed:', error)
    ElMessage.error(`强制推送失败: ${error}`)
  }
  finally {
    syncLoading.value = false
  }
}

// 强制拉取远程数据
async function forcePull() {
  syncLoading.value = true

  try {
    // 确保数据库连接有效
    if (connectionStatus.value !== 'connected') {
      ElMessage.error('数据库连接未建立，请重新检查连接')
      return
    }

    // 重新建立连接以确保连接有效
    const config = await invoke('load_database_config') as any
    if (!config) {
      ElMessage.error('数据库配置不存在')
      return
    }

    await invoke('connect_database', { config })
    await invoke('check_and_initialize_tables')

    // 从远程下载数据 - 使用同步功能来获取远程数据
    const syncResult = await invoke('start_database_sync') as any
    if (!syncResult.success) {
      throw new Error(syncResult.message || '同步失败')
    }

    // 同步功能已经自动处理了数据保存，无需额外操作

    ElMessage.success('强制拉取成功')

    // 重新加载本地数据
    await todoStore.loadTodos()

    // 重新比较数据
    await compareData()
  }
  catch (error) {
    console.error('Force pull failed:', error)
    ElMessage.error(`强制拉取失败: ${error}`)
  }
  finally {
    syncLoading.value = false
  }
}

// 打开设置
function openSettings() {
  close()
  appStore.openSettings()
}

// 暴露方法给父组件
defineExpose({
  open,
})
</script>

<template>
  <ElDialog
    v-model="visible"
    title="数据同步"
    width="600px"
    :close-on-click-modal="false"
    @close="close"
  >
    <div class="sync-modal-content">
      <!-- 连接状态 -->
      <div class="connection-status mb-4">
        <div v-if="loading" class="flex items-center gap-2">
          <ElLoading size="small" />
          <span>检查数据库连接...</span>
        </div>

        <div v-else-if="connectionStatus === 'no-config'" class="text-center py-4">
          <div class="text-gray-600 mb-4">
            <svg class="w-12 h-12 mx-auto mb-2 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <p class="text-lg font-medium">
              未配置数据库连接
            </p>
            <p class="text-sm text-gray-500">
              请先配置MySQL数据库连接信息
            </p>
          </div>
          <ElButton type="primary" @click="openSettings">
            去配置
          </ElButton>
        </div>

        <div v-else-if="connectionStatus === 'failed'" class="text-center py-4">
          <div class="text-red-600 mb-4">
            <svg class="w-12 h-12 mx-auto mb-2 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <p class="text-lg font-medium">
              数据库连接失败
            </p>
            <p class="text-sm text-gray-500">
              请检查数据库配置和网络连接
            </p>
          </div>
          <div class="flex gap-2 justify-center">
            <ElButton @click="checkDatabaseStatus">
              重新检查
            </ElButton>
            <ElButton type="primary" @click="openSettings">
              检查配置
            </ElButton>
          </div>
        </div>

        <div v-else-if="connectionStatus === 'connected'" class="text-center py-4">
          <div class="text-green-600 mb-4">
            <svg class="w-12 h-12 mx-auto mb-2 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <p class="text-lg font-medium">
              数据库连接正常
            </p>
            <p class="text-sm text-gray-500">
              已连接到MySQL数据库
            </p>
          </div>
        </div>
      </div>

      <!-- 数据差异 -->
      <div v-if="connectionStatus === 'connected'" class="differences-section">
        <h3 class="text-lg font-medium mb-3">
          数据比较结果
        </h3>

        <div v-if="!hasDifferences" class="text-center py-4 text-green-600">
          <svg class="w-8 h-8 mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <p>本地和远程数据完全一致</p>
        </div>

        <div v-else class="space-y-3">
          <div
            v-for="diff in differences"
            :key="diff.type"
            class="border border-orange-200 rounded-lg p-3 bg-orange-50"
          >
            <div class="flex items-start gap-3">
              <svg class="w-5 h-5 text-orange-500 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <div class="flex-1">
                <h4 class="font-medium text-orange-800">
                  {{ diff.title }}
                </h4>
                <p class="text-sm text-orange-700 mt-1">
                  {{ diff.description }}
                </p>
                <div v-if="diff.local !== undefined && diff.remote !== undefined" class="mt-2 text-xs text-orange-600">
                  <div>本地: {{ diff.local }}</div>
                  <div>远程: {{ diff.remote }}</div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 操作按钮 -->
    <template #footer>
      <div class="flex justify-between">
        <ElButton @click="close">
          关闭
        </ElButton>

        <div v-if="connectionStatus === 'connected'" class="flex gap-2">
          <ElButton
            v-if="hasDifferences"
            type="warning"
            :loading="syncLoading"
            @click="forcePull"
          >
            强制拉取
          </ElButton>

          <ElButton
            v-if="hasDifferences"
            type="info"
            :loading="syncLoading"
            @click="forcePush"
          >
            强制推送
          </ElButton>

          <ElButton
            type="primary"
            :loading="syncLoading"
            @click="performSync"
          >
            智能同步
          </ElButton>
        </div>
      </div>
    </template>
  </ElDialog>
</template>

<style scoped>
.sync-modal-content {
  min-height: 300px;
}

.connection-status {
  border-bottom: 1px solid #e5e7eb;
  padding-bottom: 1rem;
}

.differences-section {
  border-top: 1px solid #e5e7eb;
  padding-top: 1rem;
}
</style>
