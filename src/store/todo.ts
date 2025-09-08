import type { ArchivedTodoData, TodoItem, TodoSettings } from '../types/todo'
import { invoke } from '@tauri-apps/api/core'
import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'
import { defaultTodoSettings } from '../constants/todo'
import { $confirm } from '../utils/message'
import { timeUtils } from '../utils/time'

export const useTodoStore = defineStore('todo', () => {
  // 状态
  const todos = ref<TodoItem[]>([])
  const settings = ref<TodoSettings>({ ...defaultTodoSettings })
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 计算属性
  const rootTodos = computed(() => {
    return todos.value.filter(todo => !todo.parentId)
  })

  const todoTree = computed(() => {
    const buildTree = (parentId?: string): TodoItem[] => {
      return todos.value
        .filter(todo => todo.parentId === parentId)
        .map(todo => ({
          ...todo,
          children: buildTree(todo.id),
        }))
    }
    return buildTree()
  })

  // 生成唯一ID
  const generateId = (): string => {
    return Date.now().toString(36) + Math.random().toString(36).substr(2)
  }

  // 保存待办事项到文件
  const saveTodos = async () => {
    try {
      loading.value = true
      await invoke('save_todos', { todos: todos.value })
      error.value = null
    }
    catch (err) {
      error.value = err instanceof Error ? err.message : '保存待办事项失败'
      console.error('Failed to save todos:', err)
    }
    finally {
      loading.value = false
    }
  }

  // 保存设置到文件
  const saveSettings = async () => {
    try {
      await invoke('save_settings', { settings: settings.value })
    }
    catch (err) {
      console.error('Failed to save settings:', err)
      throw err
    }
  }

  // 从文件加载待办事项
  const loadTodos = async () => {
    try {
      loading.value = true
      const todosData = await invoke('load_todos') as TodoItem[]
      todos.value = todosData || []
      error.value = null
    }
    catch (err) {
      error.value = err instanceof Error ? err.message : '加载待办事项失败'
      console.error('Failed to load todos:', err)
    }
    finally {
      loading.value = false
    }
  }

  // 从文件加载设置
  const loadSettings = async () => {
    try {
      const settingsData = await invoke('load_settings') as TodoSettings
      settings.value = { ...defaultTodoSettings, ...settingsData }
    }
    catch (err) {
      console.error('Failed to load settings:', err)
      // 如果加载设置失败，使用默认设置
      settings.value = { ...defaultTodoSettings }
    }
  }

  // 添加待办事项
  const addTodo = async (text: string, parentId?: string, deadline?: string) => {
    const newTodo: TodoItem = {
      id: generateId(),
      text: text.trim(),
      completed: false,
      createdAt: new Date().toISOString(),
      deadline,
      parentId,
    }

    todos.value.push(newTodo)
    await saveTodos()
    return newTodo
  }

  // 更新待办事项
  const updateTodo = async (id: string, updates: Partial<TodoItem>) => {
    const index = todos.value.findIndex(todo => todo.id === id)
    if (index !== -1) {
      todos.value[index] = { ...todos.value[index], ...updates }
      await saveTodos()
    }
  }

  // 切换完成状态
  const toggleTodo = async (id: string) => {
    const todo = todos.value.find(t => t.id === id)
    if (!todo)
      return

    const now = new Date().toISOString()
    const completed = !todo.completed

    // 如果是父项，需要确认是否同时操作子项
    const children = todos.value.filter(t => t.parentId === id)
    if (children.length > 0) {
      // 检查子项是否全部为完成状态
      const allChildrenCompleted = children.every(child => child.completed)
      const allChildrenIncomplete = children.every(child => !child.completed)

      // 只有当子项不全为完成状态时才弹出确认对话框
      if (completed && !allChildrenCompleted) {
        const confirmMessage = `确认将"${todo.text}"及其${children.length}个子项标记为完成？`
        await $confirm(confirmMessage)
      }
      else if (!completed && !allChildrenIncomplete) {
        const confirmMessage = `确认将"${todo.text}"及其${children.length}个子项标记为未完成？`
        await $confirm(confirmMessage)
      }

      // 更新父项和所有子项
      await updateTodo(id, {
        completed,
        completedAt: completed ? now : undefined,
      })

      for (const child of children) {
        await updateTodo(child.id, {
          completed,
          completedAt: completed ? now : undefined,
        })
      }
    }
    else {
      // 普通项目直接切换
      await updateTodo(id, {
        completed,
        completedAt: completed ? now : undefined,
      })

      // 如果是子项且标记为完成，检查是否应该自动完成父项
      if (todo.parentId && completed) {
        const parent = todos.value.find(t => t.id === todo.parentId)
        if (parent && !parent.completed) {
          const siblings = todos.value.filter(t => t.parentId === todo.parentId)
          const allSiblingsCompleted = siblings.every(sibling => sibling.completed)

          if (allSiblingsCompleted) {
            const confirmMessage = `所有子项已完成，是否自动完成父项"${parent.text}"？`
            const shouldCompleteParent = await $confirm(confirmMessage)

            if (shouldCompleteParent) {
              await updateTodo(parent.id, {
                completed: true,
                completedAt: now,
              })
            }
          }
        }
      }
    }
  }

  // 删除待办事项
  const deleteTodo = async (id: string) => {
    const todo = todos.value.find(t => t.id === id)
    if (!todo)
      return

    // 如果是父项，需要确认是否同时删除子项
    const children = todos.value.filter(t => t.parentId === id)
    if (children.length > 0) {
      const confirmMessage = `确认删除"${todo.text}"及其${children.length}个子项？`
      await $confirm(confirmMessage)
      // 删除所有子项
      for (const child of children) {
        const childIndex = todos.value.findIndex(t => t.id === child.id)
        if (childIndex !== -1) {
          todos.value.splice(childIndex, 1)
        }
      }
    }

    // 删除主项
    const index = todos.value.findIndex(t => t.id === id)
    if (index !== -1) {
      todos.value.splice(index, 1)
      await saveTodos()
    }
  }

  // 更新设置
  const updateSettings = async (newSettings: Partial<TodoSettings>) => {
    settings.value = { ...settings.value, ...newSettings }
    await saveSettings()
  }

  // 重置颜色设置为默认值
  const resetColorsToDefault = async () => {
    settings.value = { ...settings.value, colors: { ...defaultTodoSettings.colors } }
    await saveSettings()
  }

  // 归档已完成的待办事项
  const archiveCompletedTodos = async () => {
    const completedTodos = todos.value.filter((todo) => {
      return todo.completed
        && todo.completedAt
        && timeUtils.shouldArchive(todo.completedAt, settings.value.archiveDays)
    })

    if (completedTodos.length === 0)
      return

    try {
      // 加载现有归档数据
      const existingArchived = await invoke('load_archived_todos') as ArchivedTodoData
      const archivedData: ArchivedTodoData = {
        todos: [...(existingArchived?.todos || []), ...completedTodos],
        archivedAt: new Date().toISOString(),
      }

      // 保存归档数据
      await invoke('save_archived_todos', { archivedTodos: archivedData })

      // 从当前待办事项中移除已归档的项目
      const archivedIds = new Set(completedTodos.map(t => t.id))
      todos.value = todos.value.filter(todo => !archivedIds.has(todo.id))

      await saveTodos()

      console.log(`已归档 ${completedTodos.length} 个待办事项`)
    }
    catch (err) {
      console.error('归档失败:', err)
      error.value = '归档失败'
    }
  }

  // 清除归档历史
  const clearArchivedTodos = async () => {
    try {
      await invoke('clear_archived_todos')
    }
    catch (err) {
      console.error('清除归档历史失败:', err)
      error.value = '清除归档历史失败'
    }
  }

  // 获取待办事项的时间状态
  const getTodoTimeStatus = (todo: TodoItem): 'normal' | 'warning' | 'urgent' => {
    if (!todo.deadline || todo.completed)
      return 'normal'
    return timeUtils.getTimeStatus(todo.deadline)
  }

  // 获取待办事项的样式颜色
  const getTodoColor = (todo: TodoItem): string => {
    if (todo.completed) {
      return settings.value.colors.completed
    }

    const timeStatus = getTodoTimeStatus(todo)
    switch (timeStatus) {
      case 'urgent': return settings.value.colors.urgent
      case 'warning': return settings.value.colors.warning
      default: return settings.value.colors.normal
    }
  }

  // 监听数据变化并自动归档
  let archiveTimeout: ReturnType<typeof setTimeout> | null = null
  const scheduleArchiveCheck = () => {
    if (archiveTimeout)
      clearTimeout(archiveTimeout)
    archiveTimeout = setTimeout(() => {
      archiveCompletedTodos()
      archiveTimeout = null
    }, 5000) // 5秒后检查归档
  }

  // 监听待办事项变化
  watch(todos, scheduleArchiveCheck, { deep: true })

  return {
    // 状态
    todos,
    settings,
    loading,
    error,

    // 计算属性
    rootTodos,
    todoTree,

    // 方法
    addTodo,
    updateTodo,
    toggleTodo,
    deleteTodo,
    updateSettings,
    resetColorsToDefault,
    loadTodos,
    loadSettings,
    saveTodos,
    saveSettings,
    archiveCompletedTodos,
    clearArchivedTodos,
    getTodoTimeStatus,
    getTodoColor,
  }
})
