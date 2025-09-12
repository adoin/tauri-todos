<script setup lang="ts">
import type { TodoItem, TodoTimeStatus } from '../types/todo'
import { ElButton, ElCheckbox, ElDatePicker, ElDialog, ElInput, ElMessageBox } from 'element-plus'
import { onMounted, ref } from 'vue'
import { useAppStore } from '../store/app'
import { useTodoStore } from '../store/todo'
import { timeUtils } from '../utils/time'

const todoStore = useTodoStore()
const appStore = useAppStore()
const newTodoText = ref('')
const editingId = ref<string | null>(null)
const editingText = ref('')

// 日期时间选择器状态
const showDatePicker = ref(false)
const selectedTodoId = ref<string | null>(null)
const selectedDate = ref<string | null>(null)

// 输入框引用
const mainInputRef = ref<HTMLInputElement>()

// 获取待办事项的时间状态
function getTodoTimeStatus(todo: TodoItem): TodoTimeStatus {
  if (!todo.deadline || todo.completed)
    return 'normal'
  return timeUtils.getTimeStatus(todo.deadline)
}

// 获取待办事项的样式颜色
function getTodoColor(todo: TodoItem): string {
  if (todo.completed) {
    return appStore.appSettings.colors.completed
  }

  const timeStatus = getTodoTimeStatus(todo)
  switch (timeStatus) {
    case 'urgent': return appStore.appSettings.colors.urgent
    case 'warning': return appStore.appSettings.colors.warning
    default: return appStore.appSettings.colors.normal
  }
}

// 加载待办事项和设置
onMounted(async () => {
  await Promise.all([
    todoStore.loadTodos(),
    appStore.loadAppSettings(),
  ])
})

// 添加新的待办事项
async function addNewTodo() {
  if (newTodoText.value.trim()) {
    await todoStore.addTodo(newTodoText.value)
    newTodoText.value = ''
  }
}

// 添加子项
async function addChildTodo(parentId: string) {
  try {
    const { value: text } = await ElMessageBox.prompt('请输入子任务内容：', '添加子任务', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      inputPattern: /.+/,
      inputErrorMessage: '请输入任务内容',
    })
    if (text && text.trim()) {
      await todoStore.addTodo(text, parentId)
    }
  }
  catch {
    // 用户取消输入
  }
}

// 开始编辑
function startEdit(todo: TodoItem) {
  editingId.value = todo.id
  editingText.value = todo.text
}

// 保存编辑
async function saveEdit() {
  if (editingId.value && editingText.value.trim()) {
    await todoStore.updateTodo(editingId.value, { text: editingText.value.trim() })
  }
  cancelEdit()
}

// 取消编辑
function cancelEdit() {
  editingId.value = null
  editingText.value = ''
}

// 设置截止时间
function setDeadline(todoId: string) {
  const currentTodo = todoStore.todos.data.find((t: TodoItem) => t.id === todoId)
  selectedTodoId.value = todoId
  if (currentTodo?.deadline) {
    // 将 ISO 字符串转换为 Date 对象，然后格式化为 YYYY-MM-DD HH:mm 格式
    const date = new Date(currentTodo.deadline)
    const year = date.getFullYear()
    const month = String(date.getMonth() + 1).padStart(2, '0')
    const day = String(date.getDate()).padStart(2, '0')
    const hours = String(date.getHours()).padStart(2, '0')
    const minutes = String(date.getMinutes()).padStart(2, '0')
    selectedDate.value = `${year}-${month}-${day} ${hours}:${minutes}`
  }
  else {
    selectedDate.value = null
  }
  showDatePicker.value = true
}

// 确认设置截止时间
async function confirmDeadline() {
  if (selectedTodoId.value) {
    let deadline: string | undefined
    if (selectedDate.value) {
      // selectedDate.value 是字符串格式，转换为 ISO 字符串
      deadline = new Date(selectedDate.value).toISOString()
    }
    await todoStore.updateTodo(selectedTodoId.value, { deadline })
  }
  closeDatePicker()
}

// 清除截止时间
async function clearDeadline() {
  if (selectedTodoId.value) {
    await todoStore.updateTodo(selectedTodoId.value, { deadline: undefined })
  }
  closeDatePicker()
}

// 关闭日期选择器
function closeDatePicker() {
  showDatePicker.value = false
  selectedTodoId.value = null
  selectedDate.value = null
}

// 获取时间显示文本
function getTimeDisplay(todo: TodoItem): string {
  if (!todo.deadline)
    return ''

  const status = getTodoTimeStatus(todo)
  const timeStr = timeUtils.formatTime(todo.deadline)

  switch (status) {
    case 'urgent': return `🔴 ${timeStr}`
    case 'warning': return `🟡 ${timeStr}`
    default: return `⏰ ${timeStr}`
  }
}

// 处理键盘事件
function handleKeyDown(event: Event | KeyboardEvent) {
  const keyboardEvent = event as KeyboardEvent
  if (keyboardEvent.key === 'Enter') {
    if (editingId.value) {
      saveEdit()
    }
    else if (newTodoText.value.trim()) {
      addNewTodo()
    }
  }
  else if (keyboardEvent.key === 'Escape') {
    if (editingId.value) {
      cancelEdit()
    }
  }
}

// 输入框点击处理
function handleInputClick(event: MouseEvent) {
  // 确保事件不被阻止
  event.stopPropagation()
  event.preventDefault()
  // 强制获得焦点
  const target = event.target as HTMLInputElement
  if (target) {
    // 使用setTimeout确保在事件处理完成后获得焦点
    setTimeout(() => {
      target.focus()
    }, 0)
  }
}

// 删除待办事项（带确认）
async function deleteTodoWithConfirm(todoId: string) {
  try {
    await ElMessageBox.confirm('确定要删除这个待办事项吗？', '确认删除', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    })
    await todoStore.deleteTodo(todoId)
  }
  catch {
    // 用户取消删除
  }
}
</script>

<template>
  <div class="p-4 max-h-full overflow-y-auto rounded-lg">
    <!-- 添加新待办事项 -->
    <div class="flex gap-2 mb-4 pb-4 border-b border-white/20 items-center">
      <ElInput
        ref="mainInputRef"
        v-model="newTodoText"
        placeholder="添加新的待办事项..."
        class="flex-1 px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-100"
        @keydown="handleKeyDown"
        @click="handleInputClick"
        @mousedown="handleInputClick"
      />
      <ElButton
        type="primary"
        :disabled="!newTodoText.trim()"
        @click="addNewTodo"
      >
        添加
      </ElButton>
    </div>

    <!-- 待办事项列表 -->
    <div class="min-h-50">
      <div v-if="todoStore.loading" class="text-center py-10 px-5 text-gray-500 text-sm">
        加载中...
      </div>

      <div v-else-if="todoStore.error" class="text-center py-10 px-5 text-red-500 text-sm">
        {{ todoStore.error }}
      </div>

      <div v-else-if="todoStore.todoTree.length === 0" class="text-center py-10 px-5 text-gray-500 text-sm">
        暂无待办事项
      </div>

      <div v-else class="space-y-2">
        <!-- 渲染待办事项树 -->
        <div
          v-for="todo in todoStore.todoTree"
          :key="todo.id"
          class="mb-2 p-2 bg-white/10 backdrop-blur-sm rounded-md transition-colors group shadow-lg todo-item"
          :class="{ 'opacity-60': todo.completed }"
        >
          <!-- 待办事项内容 -->
          <div class="flex items-start gap-2">
            <!-- 完成状态复选框 -->
            <ElCheckbox
              :model-value="todo.completed"
              class="cursor-pointer"
              @change="todoStore.toggleTodo(todo.id)"
            />

            <!-- 文本内容 -->
            <div class="flex-1 flex flex-col gap-1 pt-1">
              <ElInput
                v-if="editingId === todo.id"
                v-model="editingText"
                class="px-2 py-1 border border-blue-500 rounded text-sm focus:outline-none focus:ring-2 focus:ring-blue-200"
                @keydown="handleKeyDown"
                @click="handleInputClick"
              />
              <span
                v-else
                class="text-sm leading-relaxed cursor-pointer rounded px-1 py-0.5 text-shadow-sm"
                :style="{
                  color: getTodoColor(todo),
                  textDecoration: todo.completed ? 'line-through' : 'none',
                  textShadow: '0 1px 2px rgba(0, 0, 0, 0.8), 0 0 4px rgba(0, 0, 0, 0.5)',
                }"
                @dblclick="startEdit(todo)"
              >
                {{ todo.text }}
              </span>

              <!-- 时间显示 -->
              <div v-if="todo.deadline && editingId !== todo.id" class="text-xs opacity-80" style="text-shadow: 0 1px 2px rgba(0, 0, 0, 0.8), 0 0 4px rgba(0, 0, 0, 0.5);">
                {{ getTimeDisplay(todo) }}
              </div>

              <!-- 完成时间显示 -->
              <div v-if="todo.completed && todo.completedAt" class="text-xs opacity-60 text-gray-500" style="text-shadow: 0 1px 2px rgba(0, 0, 0, 0.8), 0 0 4px rgba(0, 0, 0, 0.5);">
                完成于: {{ timeUtils.formatTime(todo.completedAt) }}
              </div>
            </div>

            <!-- 操作按钮 -->
            <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity ml-auto items-center">
              <template v-if="editingId === todo.id">
                <ElButton size="small" type="success" @click="saveEdit">
                  保存
                </ElButton>
                <ElButton size="small" @click="cancelEdit">
                  取消
                </ElButton>
              </template>

              <template v-else>
                <ElButton
                  size="small"
                  type="primary"
                  plain
                  title="添加子项"
                  @click="addChildTodo(todo.id)"
                >
                  ➕ 子项
                </ElButton>
                <ElButton
                  size="small"
                  type="danger"
                  title="删除"
                  plain
                  @click="deleteTodoWithConfirm(todo.id)"
                >
                  删除
                </ElButton>
                <ElButton
                  size="small"
                  title="设置截止时间"
                  type="success"
                  plain
                  @click="setDeadline(todo.id)"
                >
                  ⏰
                </ElButton>
                <ElButton
                  size="small"
                  title="编辑"
                  type="primary"
                  plain
                  @click="startEdit(todo)"
                >
                  ✏️
                </ElButton>
              </template>
            </div>
          </div>

          <!-- 递归渲染子项 -->
          <div v-if="todo.children && todo.children.length > 0" class="ml-5 border-l-2 border-white/20 pl-3 mt-2">
            <div
              v-for="child in todo.children"
              :key="child.id"
              class="mb-2 p-2 bg-white/5 backdrop-blur-sm rounded-md transition-colors group ml-0 shadow-md todo-item-child"
              :class="{ 'opacity-60': child.completed }"
            >
              <div class="flex items-start gap-2">
                <ElCheckbox
                  :model-value="child.completed"
                  class="cursor-pointer"
                  @change="todoStore.toggleTodo(child.id)"
                />

                <div class="flex-1 flex flex-col gap-1 pt-1">
                  <ElInput
                    v-if="editingId === child.id"
                    v-model="editingText"
                    class="px-2 py-1 border border-blue-500 rounded text-sm focus:outline-none focus:ring-2 focus:ring-blue-200"
                    @keydown="handleKeyDown"
                    @click="handleInputClick"
                  />
                  <span
                    v-else
                    class="text-sm leading-relaxed cursor-pointer hover:rounded px-1 py-0.5"
                    :style="{
                      color: getTodoColor(child),
                      textDecoration: child.completed ? 'line-through' : 'none',
                      textShadow: '0 1px 2px rgba(0, 0, 0, 0.8), 0 0 4px rgba(0, 0, 0, 0.5)',
                    }"
                    @dblclick="startEdit(child)"
                  >
                    {{ child.text }}
                  </span>

                  <div v-if="child.deadline && editingId !== child.id" class="text-xs opacity-80" style="text-shadow: 0 1px 2px rgba(0, 0, 0, 0.8), 0 0 4px rgba(0, 0, 0, 0.5);">
                    {{ getTimeDisplay(child) }}
                  </div>

                  <div v-if="child.completed && child.completedAt" class="text-xs opacity-60 text-gray-500" style="text-shadow: 0 1px 2px rgba(0, 0, 0, 0.8), 0 0 4px rgba(0, 0, 0, 0.5);">
                    完成于: {{ timeUtils.formatTime(child.completedAt) }}
                  </div>
                </div>

                <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity ml-auto items-center">
                  <template v-if="editingId === child.id">
                    <ElButton size="small" type="success" @click="saveEdit">
                      保存
                    </ElButton>
                    <ElButton size="small" @click="cancelEdit">
                      取消
                    </ElButton>
                  </template>

                  <template v-else>
                    <ElButton
                      size="small"
                      type="danger"
                      plain
                      title="删除"
                      @click="deleteTodoWithConfirm(child.id)"
                    >
                      删除
                    </ElButton>
                    <ElButton
                      size="small"
                      type="success"
                      plain
                      title="设置截止时间"
                      @click="setDeadline(child.id)"
                    >
                      ⏰
                    </ElButton>
                    <ElButton
                      size="small"
                      type="primary"
                      plain
                      title="编辑"
                      @click="startEdit(child)"
                    >
                      ✏️
                    </ElButton>
                  </template>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 日期时间选择器对话框 -->
    <ElDialog
      v-model="showDatePicker"
      title="设置截止时间"
      width="400px"
      :before-close="closeDatePicker"
    >
      <div class="py-5">
        <ElDatePicker
          v-model="selectedDate"
          type="datetime"
          placeholder="选择截止日期和时间"
          format="YYYY-MM-DD HH:mm"
          value-format="YYYY-MM-DD HH:mm"
          style="width: 100%"
          :disabled-date="(time: Date) => time.getTime() < Date.now() - 24 * 60 * 60 * 1000"
        />
      </div>

      <template #footer>
        <div class="flex justify-end gap-3">
          <ElButton @click="closeDatePicker">
            取消
          </ElButton>
          <ElButton type="danger" @click="clearDeadline">
            清除时间
          </ElButton>
          <ElButton type="primary" @click="confirmDeadline">
            确认
          </ElButton>
        </div>
      </template>
    </ElDialog>
  </div>
</template>

<style scoped>
.todo-item {
  border: 1px solid var(--todo-border-color);
}

.todo-item:hover {
  background-color: rgba(255, 255, 255, 0.2) !important;
}

.todo-item-child {
  border: 1px solid var(--todo-border-color);
}

.todo-item-child:hover {
  background-color: rgba(255, 255, 255, 0.2) !important;
}
</style>
