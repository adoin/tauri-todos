<script setup lang="ts">
import { ElButton, ElColorPicker, ElDialog, ElForm, ElFormItem, ElMessage, ElMessageBox, ElSlider, ElSwitch } from 'element-plus'
import { useAppStore } from '../store/app'
import { useTodoStore } from '../store/todo'

const appStore = useAppStore()
const todoStore = useTodoStore()

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
</script>

<template>
  <ElDialog
    v-model="appStore.isSettingsOpen"
    title="Ton 设置"
    width="500px"
    :before-close="closeSettings"
  >
    <ElForm label-width="120px" label-position="left">
      <!-- 外观设置 -->
      <div class="mb-6">
        <h3 class="text-lg font-semibold mb-4 pb-2 border-b border-gray-200">
          外观设置
        </h3>

        <ElFormItem label="窗口透明度">
          <div class="flex items-center gap-3">
            <ElSlider
              v-model="appStore.windowConfig.opacity"
              :min="0.1"
              :max="1"
              :step="0.1"
              :format-tooltip="(val: number) => `${Math.round(val * 100)}%`"
              style="flex: 1"
              @change="updateOpacity"
            />
            <span class="text-sm text-gray-500 min-w-12">{{ Math.round(appStore.windowConfig.opacity * 100) }}%</span>
          </div>
        </ElFormItem>

        <ElFormItem label="圆角半径">
          <div class="flex items-center gap-3">
            <ElSlider
              v-model="appStore.windowConfig.borderRadius"
              :min="0"
              :max="20"
              :step="1"
              :format-tooltip="(val: number) => `${val}px`"
              style="flex: 1"
              @change="updateBorderRadius"
            />
            <span class="text-sm text-gray-500 min-w-12">{{ appStore.windowConfig.borderRadius }}px</span>
          </div>
        </ElFormItem>

        <ElFormItem label="边框宽度">
          <div class="flex items-center gap-3">
            <ElSlider
              v-model="appStore.windowConfig.borderWidth"
              :min="0"
              :max="5"
              :step="1"
              :format-tooltip="(val: number) => `${val}px`"
              style="flex: 1"
              @change="updateBorderWidth"
            />
            <span class="text-sm text-gray-500 min-w-12">{{ appStore.windowConfig.borderWidth }}px</span>
          </div>
        </ElFormItem>

        <ElFormItem label="边框颜色">
          <ElColorPicker
            v-model="appStore.windowConfig.borderColor"
            @change="updateBorderColor"
          />
        </ElFormItem>
      </div>

      <!-- 待办事项设置 -->
      <div class="mb-6">
        <h3 class="text-lg font-semibold mb-4 pb-2 border-b border-gray-200">
          待办事项设置
        </h3>

        <ElFormItem label="归档时间（天）">
          <div class="flex items-center gap-3">
            <ElSlider
              :model-value="todoStore.settings.archiveDays"
              :min="1"
              :max="365"
              :step="1"
              :format-tooltip="(val: number) => `${val}天`"
              style="flex: 1"
              @change="updateArchiveDays"
            />
            <span class="text-sm text-gray-500 min-w-12">{{ todoStore.settings.archiveDays }}天</span>
          </div>
        </ElFormItem>

        <ElFormItem label="清除归档历史">
          <ElButton
            type="danger"
            size="small"
            @click="clearArchivedTodos"
          >
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
            :model-value="todoStore.settings.colors.normal"
            @change="(color: string) => updateTodoColor('normal', color)"
          />
        </ElFormItem>

        <ElFormItem label="警告状态（24小时内）">
          <ElColorPicker
            :model-value="todoStore.settings.colors.warning"
            @change="(color: string) => updateTodoColor('warning', color)"
          />
        </ElFormItem>

        <ElFormItem label="紧急状态（1小时内）">
          <ElColorPicker
            :model-value="todoStore.settings.colors.urgent"
            @change="(color: string) => updateTodoColor('urgent', color)"
          />
        </ElFormItem>

        <ElFormItem label="已完成">
          <ElColorPicker
            :model-value="todoStore.settings.colors.completed"
            @change="(color: string) => updateTodoColor('completed', color)"
          />
        </ElFormItem>

        <ElFormItem label="背景颜色">
          <ElColorPicker
            :model-value="todoStore.settings.colors.background"
            @change="(color: string) => updateTodoColor('background', color)"
          />
        </ElFormItem>

        <ElFormItem label="边框颜色">
          <ElColorPicker
            :model-value="todoStore.settings.colors.border"
            @change="(color: string) => updateTodoColor('border', color)"
          />
        </ElFormItem>

        <ElFormItem label="悬停颜色">
          <ElColorPicker
            :model-value="todoStore.settings.colors.hover"
            @change="(color: string) => updateTodoColor('hover', color)"
          />
        </ElFormItem>
      </div>

      <!-- 行为设置 -->
      <div class="mb-6">
        <h3 class="text-lg font-semibold mb-4 pb-2 border-b border-gray-200">
          行为设置
        </h3>

        <ElFormItem label="透明背景">
          <ElSwitch
            v-model="appStore.isTransparent"
            @change="appStore.toggleTransparency"
          />
        </ElFormItem>
      </div>
    </ElForm>

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
</template>
