<script setup lang="ts">
import { Loading } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import { ElButton, ElDialog, ElIcon, ElMessage } from 'element-plus'
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
const comparing = ref(false)
const localData = ref<any>(null)
const remoteData = ref<any>(null)
const currentStep = ref('')

// 计算属性
const hasDifferences = computed(() => differences.value.length > 0)
const canSmartSync = computed(() =>
  !comparing.value
  && !loading.value
  && connectionStatus.value === 'connected'
  && differences.value.length > 0,
)

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
  comparing.value = false
  localData.value = null
  remoteData.value = null
  currentStep.value = ''
}

// 检查数据库状态
async function checkDatabaseStatus() {
  loading.value = true
  connectionStatus.value = 'checking'

  try {
    // 1. 检查是否有数据库配置
    currentStep.value = '检查数据库配置...'
    const config = await invoke('load_database_config') as any
    if (!config) {
      connectionStatus.value = 'no-config'
      hasConfig.value = false
      currentStep.value = ''
      return
    }

    hasConfig.value = true

    // 2. 测试数据库连接
    currentStep.value = '测试数据库连接...'
    const isConnected = await invoke('test_database_connection', { config })
    if (!isConnected) {
      connectionStatus.value = 'failed'
      currentStep.value = ''
      return
    }

    connectionStatus.value = 'connected'

    // 3. 建立连接并检查表结构
    currentStep.value = '建立数据库连接...'
    await invoke('connect_database', { config })

    currentStep.value = '检查表结构...'
    await invoke('check_and_initialize_tables')

    // 4. 比较数据
    currentStep.value = ''
    await compareData()

    console.log('数据库连接检查完成，状态:', connectionStatus.value)
  }
  catch (error) {
    console.error('Database check failed:', error)
    connectionStatus.value = 'failed'
    currentStep.value = ''
    ElMessage.error('数据库连接失败，请检查配置')
  }
  finally {
    loading.value = false
  }
}

// 比较本地和远程数据
async function compareData() {
  comparing.value = true
  try {
    // 获取本地数据
    currentStep.value = '获取本地数据...'
    const localTodos = await invoke('load_todos') as any
    const localSettings = await invoke('load_app_settings') as any

    localData.value = {
      todos: localTodos.data || [],
      settings: localSettings,
      lastUpdate: localTodos.lastUpdate,
    }

    // 获取远程数据
    currentStep.value = '获取远程数据...'
    const remoteDataResult = await invoke('get_remote_data_for_comparison') as any
    remoteData.value = {
      todos: remoteDataResult.todos || [],
      settings: remoteDataResult.settings || {},
      lastUpdate: remoteDataResult.lastUpdate,
    }

    // 比较数据差异
    currentStep.value = '分析数据差异...'
    const diffs = compareDataDifferences(
      localData.value.todos,
      remoteData.value.todos,
      localData.value.settings,
      remoteData.value.settings,
    )
    differences.value = diffs

    currentStep.value = ''
  }
  catch (error) {
    console.error('Data comparison failed:', error)
    currentStep.value = ''
    ElMessage.error('数据比较失败')
  }
  finally {
    comparing.value = false
  }
}

// 比较数据差异
function compareDataDifferences(localTodos: any, remoteTodos: any, _localSettings: any, _remoteSettings: any) {
  const diffs: any[] = []

  // 创建本地和远程待办事项的映射
  const localMap = new Map()
  const remoteMap = new Map()

  // 填充本地数据映射
  localTodos.forEach((todo: any) => {
    localMap.set(todo.id, todo)
  })

  // 填充远程数据映射
  remoteTodos.forEach((todo: any) => {
    remoteMap.set(todo.id, todo)
  })

  // 找出所有唯一的ID
  const allIds = new Set([...localMap.keys(), ...remoteMap.keys()])

  // 比较每个待办事项
  allIds.forEach((id) => {
    const localTodo = localMap.get(id)
    const remoteTodo = remoteMap.get(id)

    if (!localTodo && remoteTodo) {
      // 远程有，本地没有
      diffs.push({
        type: 'missing_local',
        id,
        local: null,
        remote: remoteTodo,
        title: '本地缺失待办事项',
        description: `远程存在但本地缺失: "${remoteTodo.text}"`,
      })
    }
    else if (localTodo && !remoteTodo) {
      // 本地有，远程没有
      diffs.push({
        type: 'missing_remote',
        id,
        local: localTodo,
        remote: null,
        title: '远程缺失待办事项',
        description: `本地存在但远程缺失: "${localTodo.text}"`,
      })
    }
    else if (localTodo && remoteTodo) {
      // 两边都有，比较内容
      const localText = localTodo.text || ''
      const remoteText = remoteTodo.text || ''
      const localCompleted = localTodo.completed || false
      const remoteCompleted = remoteTodo.completed || false

      if (localText !== remoteText || localCompleted !== remoteCompleted) {
        diffs.push({
          type: 'content_diff',
          id,
          local: localTodo,
          remote: remoteTodo,
          title: '内容不一致',
          description: `待办事项内容存在差异: "${localText}" vs "${remoteText}"`,
        })
      }
    }
  })

  // 如果没有差异，添加一个表示数据一致的项目
  if (diffs.length === 0) {
    diffs.push({
      type: 'no_diff',
      title: '数据完全一致',
      description: '本地和远程数据完全相同，无需同步',
      local: localTodos.length,
      remote: remoteTodos.length,
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

// 格式化日期时间
function formatDateTime(dateString: string | undefined | null): string {
  if (!dateString) {
    return '未知时间'
  }

  try {
    const date = new Date(dateString)
    if (Number.isNaN(date.getTime())) {
      return '无效时间'
    }

    const year = date.getFullYear()
    const month = String(date.getMonth() + 1).padStart(2, '0')
    const day = String(date.getDate()).padStart(2, '0')
    const hours = String(date.getHours()).padStart(2, '0')
    const minutes = String(date.getMinutes()).padStart(2, '0')
    const seconds = String(date.getSeconds()).padStart(2, '0')

    return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`
  }
  catch {
    return '时间格式错误'
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
    width="800px"
    :close-on-click-modal="false"
    @close="close"
  >
    <template #header>
      <div class="flex items-center justify-between w-full">
        <h3 class="text-lg font-medium">
          数据同步
        </h3>
        <div class="flex items-center gap-2">
          <!-- 连接状态图标 -->
          <div v-if="connectionStatus === 'connected' && !loading && !comparing" class="flex items-center gap-1 text-green-600">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <span class="text-xs">已连接</span>
          </div>
          <div v-else-if="connectionStatus === 'failed'" class="flex items-center gap-1 text-red-600">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <span class="text-xs">连接失败</span>
          </div>
          <div v-else-if="connectionStatus === 'no-config'" class="flex items-center gap-1 text-orange-600">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <span class="text-xs">未配置</span>
          </div>
        </div>
      </div>
    </template>
    <div class="sync-modal-content">
      <!-- 未配置状态 -->
      <div v-if="connectionStatus === 'no-config'" class="text-center py-8">
        <div class="text-gray-600 mb-6">
          <svg class="w-16 h-16 mx-auto mb-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <p class="text-xl font-medium mb-2">
            未配置数据库连接
          </p>
          <p class="text-gray-500">
            请先配置MySQL数据库连接信息
          </p>
        </div>
        <ElButton type="primary" @click="openSettings">
          去配置
        </ElButton>
      </div>

      <!-- 连接失败状态 -->
      <div v-else-if="connectionStatus === 'failed'" class="text-center py-8">
        <div class="text-red-600 mb-6">
          <svg class="w-16 h-16 mx-auto mb-4 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <p class="text-xl font-medium mb-2">
            数据库连接失败
          </p>
          <p class="text-gray-500">
            请检查数据库配置和网络连接
          </p>
        </div>
        <div class="flex gap-3 justify-center">
          <ElButton @click="checkDatabaseStatus">
            重新检查
          </ElButton>
          <ElButton type="primary" @click="openSettings">
            检查配置
          </ElButton>
        </div>
      </div>

      <!-- 数据比较区域 -->
      <div v-else-if="connectionStatus === 'connected'" class="differences-section">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-medium">
            数据比较结果
          </h3>
        </div>

        <!-- 数据完全一致 -->
        <div v-if="!hasDifferences && !comparing" class="text-center py-8 text-green-600">
          <svg class="w-12 h-12 mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <p class="text-lg font-medium">
            数据完全一致
          </p>
          <p class="text-sm text-gray-500 mt-1">
            本地和远程数据完全相同，无需同步
          </p>
        </div>

        <!-- 数据差异对比 -->
        <div v-else-if="hasDifferences && !comparing" class="space-y-4">
          <div
            v-for="diff in differences"
            :key="diff.id || diff.type"
            class="border rounded-lg p-4"
            :class="{
              'border-green-200 bg-green-50': diff.type === 'no_diff',
              'border-orange-200 bg-orange-50': diff.type !== 'no_diff',
            }"
          >
            <!-- 差异标题 -->
            <div class="flex items-center gap-2 mb-3">
              <svg
                class="w-5 h-5"
                :class="{
                  'text-green-500': diff.type === 'no_diff',
                  'text-orange-500': diff.type !== 'no_diff',
                }"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  v-if="diff.type === 'no_diff'"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                />
                <path
                  v-else
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
              <h4
                class="font-medium" :class="{
                  'text-green-800': diff.type === 'no_diff',
                  'text-orange-800': diff.type !== 'no_diff',
                }"
              >
                {{ diff.title }}
              </h4>
            </div>

            <!-- 左右对比显示 -->
            <div v-if="diff.type !== 'no_diff'" class="grid grid-cols-2 gap-4">
              <!-- 本地数据 -->
              <div class="space-y-2">
                <div class="text-sm font-medium text-blue-600 border-b border-blue-200 pb-1">
                  本地数据
                  <div v-if="diff.local" class="text-xs text-gray-500 font-normal mt-1">
                    {{ formatDateTime(diff.local.lastUpdate || diff.local.createdAt) }}
                  </div>
                  <div v-else class="text-xs text-gray-500 font-normal mt-1">
                    {{ formatDateTime(localData?.lastUpdate) }}
                  </div>
                </div>
                <div v-if="diff.local" class="p-3 bg-blue-50 rounded border border-blue-200">
                  <div class="font-medium">
                    {{ diff.local.text }}
                  </div>
                  <div class="text-xs text-gray-500 mt-1">
                    状态: {{ diff.local.completed ? '已完成' : '未完成' }}
                  </div>
                </div>
                <div v-else class="p-3 bg-gray-50 rounded border border-gray-200 text-gray-500 text-center">
                  无数据
                </div>
              </div>

              <!-- 远程数据 -->
              <div class="space-y-2">
                <div class="text-sm font-medium text-green-600 border-b border-green-200 pb-1">
                  远程数据
                  <div v-if="diff.remote" class="text-xs text-gray-500 font-normal mt-1">
                    {{ formatDateTime(diff.remote.lastUpdate || diff.remote.createdAt) }}
                  </div>
                  <div v-else class="text-xs text-gray-500 font-normal mt-1">
                    {{ formatDateTime(remoteData?.lastUpdate) }}
                  </div>
                </div>
                <div v-if="diff.remote" class="p-3 bg-green-50 rounded border border-green-200">
                  <div class="font-medium">
                    {{ diff.remote.text }}
                  </div>
                  <div class="text-xs text-gray-500 mt-1">
                    状态: {{ diff.remote.completed ? '已完成' : '未完成' }}
                  </div>
                </div>
                <div v-else class="p-3 bg-gray-50 rounded border border-gray-200 text-gray-500 text-center">
                  无数据
                </div>
              </div>
            </div>

            <!-- 差异描述 -->
            <div
              v-if="diff.description" class="mt-3 text-sm" :class="{
                'text-green-700': diff.type === 'no_diff',
                'text-orange-700': diff.type !== 'no_diff',
              }"
            >
              {{ diff.description }}
            </div>
          </div>
        </div>

        <!-- 比较中状态 -->
        <div v-else-if="comparing || loading" class="text-center py-8">
          <ElIcon class="is-loading text-4xl text-blue-600">
            <Loading />
          </ElIcon>
          <p class="text-gray-600 mt-3">
            {{ currentStep || (loading ? '正在检查数据库连接...' : '正在比较本地和远程数据...') }}
          </p>
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
            :disabled="comparing || loading"
            @click="forcePull"
          >
            强制拉取
          </ElButton>

          <ElButton
            v-if="hasDifferences"
            type="info"
            :loading="syncLoading"
            :disabled="comparing || loading"
            @click="forcePush"
          >
            强制推送
          </ElButton>

          <ElButton
            type="primary"
            :loading="syncLoading"
            :disabled="!canSmartSync"
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
  min-height: 400px;
}

.differences-section {
  min-height: 300px;
}
</style>
