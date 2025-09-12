<script setup lang="ts">
import type { FormInstance, FormRules } from 'element-plus'
import type { DatabaseConfig, SyncResult, SyncStatus } from '../types/database'
import { invoke } from '@tauri-apps/api/core'
import { ElAlert, ElButton, ElDialog, ElForm, ElFormItem, ElInput, ElInputNumber, ElMessage, ElMessageBox } from 'element-plus'
import { computed, reactive, ref, watch } from 'vue'

// Props
interface Props {
  modelValue: boolean
}

// Emits
interface Emits {
  (e: 'update:modelValue', value: boolean): void
  (e: 'sync-completed', result: SyncResult): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// 响应式数据
const visible = computed({
  get: () => props.modelValue,
  set: value => emit('update:modelValue', value),
})

const formRef = ref<FormInstance>()
const testing = ref(false)
const syncing = ref(false)

const form = reactive<DatabaseConfig>({
  host: 'localhost',
  port: 3306,
  username: 'root',
  password: '',
  database: 'todo_sync',
})

const syncStatus = ref<SyncStatus>({
  isConnected: false,
  isSyncing: false,
})

// 表单验证规则
const rules: FormRules = {
  host: [
    { required: true, message: '请输入主机地址', trigger: 'blur' },
  ],
  port: [
    { required: true, message: '请输入端口号', trigger: 'blur' },
    { type: 'number', min: 1, max: 65535, message: '端口号必须在1-65535之间', trigger: 'blur' },
  ],
  username: [
    { required: true, message: '请输入用户名', trigger: 'blur' },
  ],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' },
  ],
  database: [
    { required: true, message: '请输入数据库名', trigger: 'blur' },
  ],
}

// 方法
function handleClose() {
  visible.value = false
  // 重置表单
  Object.assign(form, {
    host: 'localhost',
    port: 3306,
    username: 'root',
    password: '',
    database: 'todo_sync',
  })
  syncStatus.value = {
    isConnected: false,
    isSyncing: false,
  }
  formRef.value?.clearValidate()
}

async function testConnection() {
  if (!formRef.value)
    return

  try {
    await formRef.value.validate()
    testing.value = true
    syncStatus.value.error = undefined

    // 调用 Tauri 命令测试连接
    const result = await invoke('test_database_connection', { config: form })

    if (result) {
      syncStatus.value.isConnected = true
      ElMessage.success('数据库连接成功')
    }
    else {
      syncStatus.value.isConnected = false
      ElMessage.error('数据库连接失败')
    }
  }
  catch (error) {
    console.error('连接测试失败:', error)
    syncStatus.value.isConnected = false
    syncStatus.value.error = error instanceof Error ? error.message : '连接失败'
    ElMessage.error('连接测试失败')
  }
  finally {
    testing.value = false
  }
}

async function startSync() {
  if (!syncStatus.value.isConnected) {
    ElMessage.warning('请先测试数据库连接')
    return
  }

  try {
    await ElMessageBox.confirm(
      '开启同步将比较本地和远程数据，并同步最新版本。是否继续？',
      '确认同步',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      },
    )

    syncing.value = true
    syncStatus.value.isSyncing = true

    // 保存数据库配置
    await invoke('save_database_config', { config: form })

    // 建立数据库连接
    await invoke('connect_database', { config: form })

    // 检查并初始化表结构
    const tableMessage = await invoke('check_and_initialize_tables') as string
    ElMessage.info(tableMessage)

    // 开始同步
    const result = await invoke('start_database_sync') as SyncResult

    if (result.success) {
      syncStatus.value.lastSyncTime = new Date().toISOString()
      ElMessage.success(result.message)
      emit('sync-completed', result)
    }
    else {
      ElMessage.error(result.message)
    }
  }
  catch (error) {
    if (error !== 'cancel') {
      console.error('同步失败:', error)
      ElMessage.error('同步失败')
    }
  }
  finally {
    syncing.value = false
    syncStatus.value.isSyncing = false
  }
}

function formatTime(timeStr: string) {
  return new Date(timeStr).toLocaleString('zh-CN')
}

// 组件挂载时尝试加载已保存的配置
async function loadSavedConfig() {
  try {
    const savedConfig = await invoke('load_database_config') as DatabaseConfig | null
    if (savedConfig) {
      Object.assign(form, savedConfig)
      // 自动测试连接
      await testConnection()
    }
  }
  catch (error) {
    console.error('加载数据库配置失败:', error)
  }
}

// 监听对话框打开
watch(() => props.modelValue, (newVal) => {
  if (newVal) {
    loadSavedConfig()
  }
})
</script>

<template>
  <ElDialog
    v-model="visible"
    title="数据库同步配置"
    width="500px"
    :close-on-click-modal="false"
    @close="handleClose"
  >
    <ElForm
      ref="formRef"
      :model="form"
      :rules="rules"
      label-width="100px"
      label-position="left"
    >
      <ElFormItem label="主机地址" prop="host">
        <ElInput
          v-model="form.host"
          placeholder="localhost"
          clearable
        />
      </ElFormItem>

      <ElFormItem label="端口" prop="port">
        <ElInputNumber
          v-model="form.port"
          :min="1"
          :max="65535"
          placeholder="3306"
          style="width: 100%"
        />
      </ElFormItem>

      <ElFormItem label="用户名" prop="username">
        <ElInput
          v-model="form.username"
          placeholder="root"
          clearable
        />
      </ElFormItem>

      <ElFormItem label="密码" prop="password">
        <ElInput
          v-model="form.password"
          type="password"
          placeholder="请输入密码"
          show-password
          clearable
        />
      </ElFormItem>

      <ElFormItem label="数据库名" prop="database">
        <ElInput
          v-model="form.database"
          placeholder="todo_sync"
          clearable
        />
      </ElFormItem>
    </ElForm>

    <!-- 连接状态显示 -->
    <div v-if="syncStatus.isConnected" class="connection-status">
      <ElAlert
        title="数据库连接成功"
        type="success"
        :closable="false"
        show-icon
      />
      <div v-if="syncStatus.lastSyncTime" class="sync-info">
        <p>最后同步时间: {{ formatTime(syncStatus.lastSyncTime) }}</p>
      </div>
    </div>

    <div v-if="syncStatus.error" class="connection-status">
      <ElAlert
        :title="syncStatus.error"
        type="error"
        :closable="false"
        show-icon
      />
    </div>

    <template #footer>
      <div class="dialog-footer">
        <ElButton @click="handleClose">
          取消
        </ElButton>
        <ElButton
          type="primary"
          :loading="testing"
          @click="testConnection"
        >
          测试连接
        </ElButton>
        <ElButton
          type="success"
          :loading="syncing"
          :disabled="!syncStatus.isConnected"
          @click="startSync"
        >
          开启同步
        </ElButton>
      </div>
    </template>
  </ElDialog>
</template>

<style scoped>
.connection-status {
  margin-top: 16px;
}

.sync-info {
  margin-top: 8px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
