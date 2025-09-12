<script setup lang="ts">
import { ElButton, ElColorPicker, ElDialog, ElForm, ElFormItem, ElInput, ElMessage, ElMessageBox, ElOption, ElSelect, ElSlider, ElSwitch } from 'element-plus'
import { ref } from 'vue'
import { useAppStore } from '../store/app'
import { useTodoStore } from '../store/todo'
import DatabaseConfigModal from './DatabaseConfigModal.vue'

const appStore = useAppStore()
const todoStore = useTodoStore()

// 数据库配置模态框状态
const isDatabaseConfigOpen = ref(false)

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
  await todoStore.updateSettings({ archiveDays: val })
}

async function updateTodoColor(colorKey: string, color: string | null) {
  if (color) {
    const colors = { ...todoStore.settings.colors, [colorKey]: color }
    await todoStore.updateSettings({ colors })
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
    await todoStore.resetColorsToDefault()
    ElMessage.success('颜色设置已恢复为默认值')
  }
  catch {
    // 用户取消操作
  }
}

function updateOpacity(value: number | number[]) {
  const val = Array.isArray(value) ? value[0] : value
  appStore.updateWindowConfig({ opacity: val })
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
  console.log('同步完成:', result)
  if (result.success) {
    // 关闭同步配置的弹窗
    isDatabaseConfigOpen.value = false
  }
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

          <ElFormItem label="窗口不透明度">
            <div class="flex items-center gap-3 w-full">
              <ElSlider
                v-model="appStore.windowConfig.opacity" :min="0.1" :max="1" :step="0.1"
                :format-tooltip="(val: number) => `${Math.round(val * 100)}%`" style="flex: 1"
                @change="updateOpacity"
              />
              <span class="text-sm text-gray-500 min-w-12">{{ Math.round(appStore.windowConfig.opacity * 100) }}%</span>
            </div>
          </ElFormItem>

          <ElFormItem label="圆角半径">
            <div class="flex items-center gap-3">
              <ElInput
                v-model.number="appStore.windowConfig.borderRadius"
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
                v-model.number="appStore.windowConfig.borderWidth"
                type="number"
                :min="0"
                :max="5"
                style="width: 120px"
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
                v-model.number="todoStore.settings.archiveDays"
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
              :model-value="todoStore.settings.colors.normal"
              @change="(color: string | null) => updateTodoColor('normal', color)"
            />
          </ElFormItem>

          <ElFormItem label="警告状态（24小时内）">
            <ElColorPicker
              show-alpha
              :predefine="predefineColors"
              :model-value="todoStore.settings.colors.warning"
              @change="(color: string | null) => updateTodoColor('warning', color)"
            />
          </ElFormItem>

          <ElFormItem label="紧急状态（1小时内）">
            <ElColorPicker
              show-alpha
              :predefine="predefineColors"
              :model-value="todoStore.settings.colors.urgent"
              @change="(color: string | null) => updateTodoColor('urgent', color)"
            />
          </ElFormItem>

          <ElFormItem label="已完成">
            <ElColorPicker
              show-alpha
              :predefine="predefineColors"
              :model-value="todoStore.settings.colors.completed"
              @change="(color: string | null) => updateTodoColor('completed', color)"
            />
          </ElFormItem>

          <ElFormItem label="背景颜色">
            <ElColorPicker
              show-alpha
              :predefine="predefineColors"
              :model-value="todoStore.settings.colors.background"
              @change="(color: string | null) => updateTodoColor('background', color)"
            />
          </ElFormItem>

          <ElFormItem label="边框颜色">
            <ElColorPicker
              show-alpha
              :predefine="predefineColors"
              :model-value="todoStore.settings.colors.border"
              @change="(color: string | null) => updateTodoColor('border', color)"
            />
          </ElFormItem>

          <ElFormItem label="窗口边框颜色">
            <ElColorPicker
              v-model="appStore.windowConfig.borderColor" show-alpha
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
        </div>

        <!-- 行为设置 -->
        <div class="mb-6">
          <h3 class="text-lg font-semibold mb-4 pb-2 border-b border-gray-200">
            行为设置
          </h3>

          <ElFormItem label="背景模式">
            <div class="flex items-center gap-2">
              <ElSwitch v-model="appStore.isTransparent" />
              <span class="text-sm text-gray-600">
                {{ appStore.isTransparent ? '透明背景' : '纯色背景' }}
              </span>
            </div>
          </ElFormItem>

          <ElFormItem label="界面语言">
            <ElSelect v-model="appStore.locale" style="width: 200px" @change="appStore.updateLocale">
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
