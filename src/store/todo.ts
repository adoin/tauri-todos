import type { ArchivedTodoData, TodoData, TodoItem } from '../types/todo'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { writeFile } from '@tauri-apps/plugin-fs'
import { ElMessage } from 'element-plus'
import { defineStore } from 'pinia'
import { v4 as uuidv4 } from 'uuid'
import { computed, ref, watch } from 'vue'
import { $confirm } from '../utils/message'
import { timeUtils } from '../utils/time'
import { useAppStore } from './app'
import { useSyncStore } from './sync'

export const useTodoStore = defineStore('todo', () => {
  // 状态
  const todos = ref<TodoData>({
    data: [],
    lastUpdate: new Date().toISOString(),
    source: 'manual',
  })
  const appStore = useAppStore()
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 同步相关
  const syncStore = useSyncStore()
  let syncTimeout: ReturnType<typeof setTimeout> | null = null

  // 计算属性
  const rootTodos = computed(() => {
    return todos.value.data.filter(todo => !todo.parentId)
  })

  const todoTree = computed(() => {
    const buildTree = (parentId?: string): TodoItem[] => {
      return (todos.value?.data ?? [])
        .filter(todo => todo.parentId === parentId)
        .map(todo => ({
          ...todo,
          children: buildTree(todo.id),
        }))
    }
    return buildTree()
  })

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

  // 从文件加载待办事项
  const loadTodos = async () => {
    try {
      loading.value = true
      const todosData = await invoke('load_todos') as TodoData
      if (todosData) {
        todos.value = todosData
      }
      else {
        // 兼容旧格式，如果是数组则转换为新格式
        const oldData = await invoke('load_todos') as TodoItem[]
        if (Array.isArray(oldData)) {
          todos.value = {
            data: oldData,
            lastUpdate: new Date().toISOString(),
            source: 'manual',
          }
        }
        else {
          todos.value = {
            data: [],
            lastUpdate: new Date().toISOString(),
            source: 'manual',
          }
        }
      }
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

  // 添加待办事项
  const addTodo = async (text: string, parentId?: string, deadline?: string) => {
    const newTodo: TodoItem = {
      id: uuidv4(),
      text: text.trim(),
      completed: false,
      createdAt: new Date().toISOString(),
      deadline,
      parentId,
    }

    todos.value.data.push(newTodo)
    todos.value.lastUpdate = new Date().toISOString()
    todos.value.source = 'manual'
    await saveTodos()
    return newTodo
  }

  // 更新待办事项
  const updateTodo = async (id: string, updates: Partial<TodoItem>) => {
    const index = todos.value.data.findIndex(todo => todo.id === id)
    if (index !== -1) {
      todos.value.data[index] = { ...todos.value.data[index], ...updates }
      todos.value.lastUpdate = new Date().toISOString()
      todos.value.source = 'manual'
      await saveTodos()
    }
  }

  // 切换完成状态
  const toggleTodo = async (id: string) => {
    const todo = todos.value.data.find(t => t.id === id)
    if (!todo)
      return

    const now = new Date().toISOString()
    const completed = !todo.completed

    // 如果是父项，需要根据子项状态决定操作逻辑
    const children = todos.value.data.filter(t => t.parentId === id)
    if (children.length > 0) {
      const allChildrenCompleted = children.every(child => child.completed)

      if (completed) {
        // 勾选父项（标记为完成）
        if (!allChildrenCompleted) {
          // 存在未完成的子项，提示是否完成所有子项
          const confirmMessage = `确认将"${todo.text}"及其${children.length}个子项标记为完成？`
          try {
            await $confirm(confirmMessage)
            // 用户点击"确定"，同时完成父项和所有子项
            await updateTodo(id, {
              completed,
              completedAt: now,
            })
            for (const child of children) {
              await updateTodo(child.id, {
                completed: true,
                completedAt: now,
              })
            }
          }
          catch {
            // 用户点击"取消"，只更新父项为完成
            await updateTodo(id, {
              completed,
              completedAt: now,
            })
          }
        }
        else {
          // 所有子项都已完成，直接完成父项
          await updateTodo(id, {
            completed,
            completedAt: now,
          })
        }
      }
      else {
        // 取消勾选父项（标记为未完成）
        if (allChildrenCompleted) {
          // 所有子项都已完成，提示是否取消完成所有子项
          const confirmMessage = `确认将"${todo.text}"及其${children.length}个子项标记为未完成？`
          try {
            await $confirm(confirmMessage)
            // 用户点击"确定"，同时取消完成父项和所有子项
            await updateTodo(id, {
              completed,
              completedAt: undefined,
            })
            for (const child of children) {
              await updateTodo(child.id, {
                completed: false,
                completedAt: undefined,
              })
            }
          }
          catch {
            // 用户点击"取消"，只取消完成父项
            await updateTodo(id, {
              completed,
              completedAt: undefined,
            })
          }
        }
        else {
          // 子项没有都选中，直接取消完成父项
          await updateTodo(id, {
            completed,
            completedAt: undefined,
          })
        }
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
        const parent = todos.value.data.find(t => t.id === todo.parentId)
        if (parent && !parent.completed) {
          const siblings = todos.value.data.filter(t => t.parentId === todo.parentId)
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
    const todo = todos.value.data.find(t => t.id === id)
    if (!todo)
      return

    // 如果是父项，需要确认是否同时删除子项
    const children = todos.value.data.filter(t => t.parentId === id)
    if (children.length > 0) {
      const confirmMessage = `确认删除"${todo.text}"及其${children.length}个子项？`
      await $confirm(confirmMessage)
      // 删除所有子项
      for (const child of children) {
        const childIndex = todos.value.data.findIndex(t => t.id === child.id)
        if (childIndex !== -1) {
          todos.value.data.splice(childIndex, 1)
        }
      }
    }

    // 删除主项
    const index = todos.value.data.findIndex(t => t.id === id)
    if (index !== -1) {
      todos.value.data.splice(index, 1)
      await saveTodos()
    }
  }

  // 设置数据来源
  const setDataSource = (source: 'manual' | 'import' | 'sync') => {
    todos.value.source = source
    todos.value.lastUpdate = new Date().toISOString()
  }

  // 归档已完成的待办事项
  const archiveCompletedTodos = async (archiveDays: number = 7) => {
    const completedTodos = todos.value.data.filter((todo) => {
      return todo.completed
        && todo.completedAt
        && timeUtils.shouldArchive(todo.completedAt, archiveDays)
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
      todos.value.data = todos.value.data.filter(todo => !archivedIds.has(todo.id))
      todos.value.lastUpdate = new Date().toISOString()
      todos.value.source = 'manual'

      await saveTodos()
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

  // 监听数据变化并自动归档
  let archiveTimeout: ReturnType<typeof setTimeout> | null = null
  const scheduleArchiveCheck = (archiveDays: number = 7) => {
    if (archiveTimeout)
      clearTimeout(archiveTimeout)
    archiveTimeout = setTimeout(() => {
      archiveCompletedTodos(archiveDays)
      archiveTimeout = null
    }, 5000) // 5秒后检查归档
  }

  // 导出待办数据
  const exportTodos = async () => {
    try {
      const exportData = {
        todos: todos.value.data,
        exportedAt: new Date().toISOString(),
        version: '1.0',
      }

      const dataStr = JSON.stringify(exportData, null, 2)

      // 生成默认文件名（包含日期时间）
      const now = new Date()
      const dateStr = now.toISOString().split('T')[0] // YYYY-MM-DD
      const timeStr = now.toTimeString().split(' ')[0].replace(/:/g, '-') // HH-MM-SS
      const defaultFileName = `todos-backup-${dateStr}-${timeStr}.json`

      // 使用Tauri的save dialog
      const filePath = await save({
        defaultPath: defaultFileName,
        filters: [
          {
            name: 'JSON Files',
            extensions: ['json'],
          },
        ],
      })

      if (filePath) {
        // 使用Tauri的fs插件写入文件
        const encoder = new TextEncoder()
        const dataBytes = encoder.encode(dataStr)
        await writeFile(filePath, dataBytes)
        ElMessage.success(`待办数据导出成功: ${filePath}`)
      }
      else {
        ElMessage.info('导出已取消')
      }
    }
    catch (error) {
      console.error('导出失败详细错误:', error)

      let errorMessage = '未知错误'
      if (error instanceof Error) {
        errorMessage = error.message
        console.error('错误堆栈:', error.stack)
      }
      else if (typeof error === 'string') {
        errorMessage = error
      }
      else if (error && typeof error === 'object') {
        errorMessage = JSON.stringify(error)
      }

      ElMessage.error(`导出失败: ${errorMessage}`)
    }
  }

  // 导入待办数据
  const importTodos = async (file: File) => {
    try {
      const text = await file.text()
      const importData = JSON.parse(text)

      // 验证数据格式
      if (!importData.todos || !Array.isArray(importData.todos)) {
        throw new Error('无效的数据格式')
      }

      const confirmMessage = `确认导入 ${importData.todos.length} 个待办事项？这将覆盖当前所有数据。`
      const confirmed = await $confirm(confirmMessage)

      if (confirmed) {
        // 备份当前数据
        const currentData = { ...todos.value }

        try {
          // 导入新数据（只导入待办事项，不覆盖设置）
          todos.value = {
            data: importData.todos,
            lastUpdate: new Date().toISOString(),
            source: 'import',
          }

          // 保存到文件
          await saveTodos()

          ElMessage.success('待办数据导入成功')
        }
        catch (error) {
          // 恢复备份数据
          todos.value = currentData
          throw error
        }
      }
    }
    catch (error) {
      console.error('导入失败:', error)
      ElMessage.error(`导入失败: ${error instanceof Error ? error.message : '未知错误'}`)
    }
  }

  // 自动同步方法
  const scheduleAutoSync = () => {
    if (!syncStore.isAutoSyncEnabled || !syncStore.isSyncAvailable) {
      return
    }

    // 清除之前的定时器
    if (syncTimeout) {
      clearTimeout(syncTimeout)
    }

    // 设置新的定时器
    syncTimeout = setTimeout(async () => {
      try {
        await syncStore.startSync()
      }
      catch (error) {
        console.error('自动同步失败:', error)
        // 不显示错误消息，避免打扰用户
      }
    }, syncStore.syncDebounceTime)
  }

  // 监听待办事项变化
  watch(() => todos.value, () => {
    scheduleArchiveCheck(appStore.appSettings.archiveDays)
    scheduleAutoSync()
  }, { deep: true })

  return {
    // 状态
    todos,
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
    loadTodos,
    saveTodos,
    archiveCompletedTodos,
    clearArchivedTodos,
    exportTodos,
    importTodos,
    setDataSource,
  }
})
