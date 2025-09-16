<script setup lang="ts">
import { ElButton, ElColorPicker, ElDialog, ElForm, ElFormItem, ElInput, ElMessage, ElMessageBox, ElOption, ElSelect, ElSwitch } from 'element-plus'
import { ref } from 'vue'
import type { FormInstance, FormRules } from 'element-plus'
import { useAppStore } from '../store/app'
import { useTodoStore } from '../store/todo'
import DatabaseConfigModal from './DatabaseConfigModal.vue'

const appStore = useAppStore()
const todoStore = useTodoStore()

// 数据库配置模态框状态
const isDatabaseConfigOpen = ref(false)

// 表单引用
const formRef = ref<FormInstance>()

// 自动同步验证规则
const autoSyncRules: FormRules = {
  autoSync: [
    {
      validator: (_rule: any, value: string, callback: any) => {
        if (!value || value === '0') {
          callback() // 0 表示不自动同步，允许
          return
        }

        // 验证格式：数字 + 单位 (m/h)
        const pattern = /^(\d+)([mh])$/
        if (!pattern.test(value)) {
          callback(new Error('格式错误，请输入如 "15m" 或 "1h" 的格式'))
          return
        }

        const match = value.match(pattern)
        if (match) {
          const num = Number.parseInt(match[1])
          const unit = match[2]

          if (unit === 'm' && (num < 1 || num > 1440)) {
            callback(new Error('分钟数必须在1-1440之间'))
            return
          }

          if (unit === 'h' && (num < 1 || num > 24)) {
            callback(new Error('小时数必须在1-24之间'))
            return
          }
        }

        callback()
      },
      trigger: 'blur',
    },
  ],
}

const predefineColors = [
  '#ff4500',
  '#ff8c00',
  '#ffd700',
  '#90ee90',
  '#00ced1',
  '#1e90ff',
  '#c71585',
  'rgba(255, 69, 0, 0.68)',
  'rgb(255, 120, 0)',
  'hsv(51, 100, 98)',
  'hsva(120, 40, 94, 0.5)',
  'hsl(181, 100%, 37%)',
  'hsla(209, 100%, 56%, 0.73)',
  '#c7158577',
]
function closeSettings() {
  appStore.closeSettings()
}

// 待办事项设置相关方法
async function updateArchiveDays(value: number | number[]) {
  const val = Array.isArray(value) ? value[0] : value
  await appStore.updateAppSettings({ archiveDays: val })
}

async function updateTodoColor(colorKey: string, color: string | null) {
  if (color) {
    const colors = { ...appStore.appSettings.colors, [colorKey]: color }
    await appStore.updateAppSettings({ colors })
  }
}

async function clearArchivedTodos() {
  try {
    await ElMessageBox.confirm('确认清空所有归档历史？此操作不可撤销。', '确认清空', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    })
    await todoStore.clearArchivedTodos()
    ElMessage.success('归档历史已清空')
  }
  catch {
    // 用户取消操作
  }
}

async function resetColorsToDefault() {
  try {
    await ElMessageBox.confirm('确认将所有颜色设置恢复为默认值？', '确认重置', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    })
    await appStore.resetColorsToDefault()
    ElMessage.success('颜色设置已恢复为默认值')
  }
  catch {
    // 用户取消操作
  }
}

function updateBorderRadius(value: number | number[]) {
  const val = Array.isArray(value) ? value[0] : value
  appStore.updateWindowConfig({ borderRadius: val })
}

function updateBorderWidth(value: number | number[]) {
  const val = Array.isArray(value) ? value[0] : value
  appStore.updateWindowConfig({ borderWidth: val })
}

function updateBorderColor(color: string | null) {
  if (color) {
    appStore.updateWindowConfig({ borderColor: color })
  }
}

// 数据库同步相关方法
function openDatabaseConfig() {
  isDatabaseConfigOpen.value = true
}

function onSyncCompleted(result: any) {
  if (result.success) {
    // 关闭同步配置的弹窗
    isDatabaseConfigOpen.value = false
  }
}

// 自动同步设置更新
async function updateAutoSync(value: string) {
  await appStore.updateAppSettings({ autoSync: value })
}
</script>

<template>
  <ElDialog v-model="appStore.isSettingsOpen" title="Ton 设置" width="500px" :before-close="closeSettings">
    <div
      class="max-h-96 overflow-y-auto pr-2 scrollbar-thin scrollbar-thumb-gray-300 scrollbar-track-gray-100 hover:scrollbar-thumb-gray-400"
    >
      <ElForm label-width="160px" label-position="left">
        <!-- 外观设置 -->
        <div class="mb-6">
          <h3 class="text-lg font-semibold mb-4 pb-2 border-b border-gray-200">
            外观设置
          </h3>

          <ElFormItem label="圆角半径">
            <div class="flex items-center gap-3">
              <ElInput
                v-model.number="appStore.appSettings.windowConfig.borderRadius"
                type="number"
                :min="0"
                :max="20"
                style="width: 120px"
                @change="(value: string) => updateBorderRadius(Number(value))"
              />
              <span class="text-sm text-gray-500">px</span>
            </div>
          </ElFormItem>

          <ElFormItem label="边框宽度">
            <div class="flex items-center gap-3">
              <ElInput
                v-model.number="appStore.appSettings.windowConfig.borderWidth"
                type="number"
                :min="0"
                :max="5"
                style="width: 120px"
                placeholder="请输入边框宽度,0表示无边框"
                @change="(value: string) => updateBorderWidth(Number(value))"
              />
              <span class="text-sm text-gray-500">px</span>
            </div>
          </ElFormItem>
        </div>

        <!-- 待办事项设置 -->
        <div class="mb-6">
          <h3 class="text-lg font-semibold mb-4 pb-2 border-b border-gray-200">
            待办事项设置
          </h3>

          <ElFormItem label="归档时间（天）">
            <div class="flex items-center gap-3">
              <ElInput
                v-model.number="appStore.appSettings.archiveDays"
                type="number"
                :min="1"
                :max="365"
                style="width: 120px"
                @change="(value: string) => updateArchiveDays(Number(value))"
              />
              <span class="text-sm text-gray-500">天</span>
            </div>
          </ElFormItem>

          <ElFormItem label="清除归档历史">
            <ElButton type="danger" size="small" @click="clearArchivedTodos">
              清空历史
            </ElButton>
          </ElFormItem>
        </div>

        <!-- 颜色主题设置 -->
        <div class="mb-6">
          <h3 class="text-lg font-semibold mb-4 pb-2 border-b border-gray-200">
            颜色主题
          </h3>

          <ElFormItem label="正常状态">
            <ElColorPicker
              show-alpha
              :predefine="predefineColors"
              :model-value="appStore.appSettings.colors.normal"
              @change="(color: string | null) => updateTodoColor('normal', color)"
            />
          </ElFormItem>

          <ElFormItem label="警告状态（24小时内）">
            <ElColorPicker
              show-alpha
              :predefine="predefineColors"
              :model-value="appStore.appSettings.colors.warning"
              @change="(color: string | null) => updateTodoColor('warning', color)"
            />
          </ElFormItem>

          <ElFormItem label="紧急状态（1小时内）">
            <ElColorPicker
              show-alpha
              :predefine="predefineColors"
              :model-value="appStore.appSettings.colors.urgent"
              @change="(color: string | null) => updateTodoColor('urgent', color)"
            />
          </ElFormItem>

          <ElFormItem label="已完成">
            <ElColorPicker
              show-alpha
              :predefine="predefineColors"
              :model-value="appStore.appSettings.colors.completed"
              @change="(color: string | null) => updateTodoColor('completed', color)"
            />
          </ElFormItem>

          <ElFormItem label="背景颜色">
            <ElColorPicker
              show-alpha
              :predefine="predefineColors"
              :model-value="appStore.appSettings.colors.background"
              @change="(color: string | null) => updateTodoColor('background', color)"
            />
          </ElFormItem>

          <ElFormItem label="边框颜色">
            <ElColorPicker
              show-alpha
              :predefine="predefineColors"
              :model-value="appStore.appSettings.colors.border"
              @change="(color: string | null) => updateTodoColor('border', color)"
            />
          </ElFormItem>

          <ElFormItem label="窗口边框颜色">
            <ElColorPicker
              v-model="appStore.appSettings.windowConfig.borderColor" show-alpha
              :predefine="predefineColors" @change="updateBorderColor"
            />
          </ElFormItem>

          <ElFormItem label="重置颜色">
            <ElButton type="warning" size="small" @click="resetColorsToDefault">
              恢复默认颜色
            </ElButton>
          </ElFormItem>
        </div>

        <!-- 数据同步设置 -->
        <div class="mb-6">
          <h3 class="text-lg font-semibold mb-4 pb-2 border-b border-gray-200">
            数据同步
          </h3>

          <ElFormItem label="MySQL 同步">
            <ElButton type="primary" @click="openDatabaseConfig">
              配置数据库同步
            </ElButton>
            <div class="text-sm text-gray-500 mt-1">
              配置 MySQL 数据库连接，实现多设备数据同步
            </div>
          </ElFormItem>

          <ElFormItem label="自动同步周期" prop="autoSync" :rules="autoSyncRules">
            <ElInput
              v-model="appStore.appSettings.autoSync"
              placeholder="0 (不自动同步) 或 15m (15分钟) 或 1h (1小时)"
              @blur="() => updateAutoSync(appStore.appSettings.autoSync || '0')"
              clearable
            >
              <template #append>
                <span class="text-xs text-gray-500">
                  0=关闭 | m=分钟 | h=小时
                </span>
              </template>
            </ElInput>
            <div class="text-sm text-gray-500 mt-1">
              设置自动同步间隔，0表示不自动同步
            </div>
          </ElFormItem>
        </div>

        <!-- 行为设置 -->
        <div class="mb-6">
          <h3 class="text-lg font-semibold mb-4 pb-2 border-b border-gray-200">
            行为设置
          </h3>

          <ElFormItem label="背景模式">
            <div class="flex items-center gap-2">
              <ElSwitch v-model="appStore.appSettings.isTransparent" />
              <span class="text-sm text-gray-600">
                {{ appStore.appSettings.isTransparent ? '透明背景' : '纯色背景' }}
              </span>
            </div>
          </ElFormItem>

          <ElFormItem label="界面语言">
            <ElSelect v-model="appStore.appSettings.locale" style="width: 200px" @change="appStore.updateLocale">
              <ElOption label="中文" value="zh-cn" />
              <ElOption label="English" value="en" />
            </ElSelect>
          </ElFormItem>
        </div>
      </ElForm>
    </div>

    <template #footer>
      <div class="flex justify-end gap-3">
        <ElButton @click="closeSettings">
          取消
        </ElButton>
        <ElButton type="primary" @click="closeSettings">
          确定
        </ElButton>
      </div>
    </template>
  </ElDialog>

  <!-- 数据库配置模态框 -->
  <DatabaseConfigModal
    v-model="isDatabaseConfigOpen"
    @sync-completed="onSyncCompleted"
  />
</template>
