import type { ArchivedTodoData, TodoData, TodoItem, TodoSettings } from '../types/todo'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { writeFile } from '@tauri-apps/plugin-fs'
import { ElMessage } from 'element-plus'
import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'
import { defaultTodoSettings } from '../constants/todo'
import { $confirm } from '../utils/message'
import { timeUtils } from '../utils/time'

export const useTodoStore = defineStore('todo', () => {
  // 状态
  const todos = ref<TodoData>({
    data: [],
    lastUpdate: new Date().toISOString(),
    source: 'manual',
  })
  const settings = ref<TodoSettings>({ ...defaultTodoSettings })
  const loading = ref(false)
  const error = ref<string | null>(null)

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

  // 从文件加载设置
  const loadSettings = async () => {
    try {
      const settingsData = await invoke('load_settings') as TodoSettings
      // 确保gitSync字段存在，如果不存在则使用默认值
      const mergedSettings = {
        ...defaultTodoSettings,
        ...settingsData,
        gitSync: {
          ...defaultTodoSettings.gitSync,
          ...(settingsData.gitSync || {}),
        },
      }
      settings.value = mergedSettings
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

  // 设置数据来源
  const setDataSource = (source: 'manual' | 'import' | 'sync') => {
    todos.value.source = source
    todos.value.lastUpdate = new Date().toISOString()
  }

  // 归档已完成的待办事项
  const archiveCompletedTodos = async () => {
    const completedTodos = todos.value.data.filter((todo) => {
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
      todos.value.data = todos.value.data.filter(todo => !archivedIds.has(todo.id))
      todos.value.lastUpdate = new Date().toISOString()
      todos.value.source = 'manual'

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

  // 导出待办数据
  const exportTodos = async () => {
    try {
      console.log('开始导出待办数据...')

      const exportData = {
        todos: todos.value.data,
        exportedAt: new Date().toISOString(),
        version: '1.0',
      }

      const dataStr = JSON.stringify(exportData, null, 2)
      console.log('数据准备完成，大小:', dataStr.length, '字符')

      // 生成默认文件名（包含日期时间）
      const now = new Date()
      const dateStr = now.toISOString().split('T')[0] // YYYY-MM-DD
      const timeStr = now.toTimeString().split(' ')[0].replace(/:/g, '-') // HH-MM-SS
      const defaultFileName = `todos-backup-${dateStr}-${timeStr}.json`
      console.log('默认文件名:', defaultFileName)

      // 使用Tauri的save dialog
      console.log('打开保存对话框...')
      const filePath = await save({
        defaultPath: defaultFileName,
        filters: [
          {
            name: 'JSON Files',
            extensions: ['json'],
          },
        ],
      })

      console.log('用户选择的文件路径:', filePath)

      if (filePath) {
        console.log('开始写入文件到:', filePath)
        // 使用Tauri的fs插件写入文件
        const encoder = new TextEncoder()
        const dataBytes = encoder.encode(dataStr)
        await writeFile(filePath, dataBytes)
        console.log('文件写入成功')
        ElMessage.success(`待办数据导出成功: ${filePath}`)
      }
      else {
        console.log('用户取消了保存')
        ElMessage.info('导出已取消')
      }
    }
    catch (error) {
      console.error('导出失败详细错误:', error)
      console.error('错误类型:', typeof error)
      console.error('错误构造函数:', error?.constructor?.name)

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

  // Git同步相关方法
  const initializeGitSync = async () => {
    try {
      loading.value = true
      const result = await invoke('initialize_git_sync', {
        repositoryUrl: settings.value.gitSync.repositoryUrl,
        authMethod: settings.value.gitSync.authMethod,
        sshKeyPath: settings.value.gitSync.sshKeyPath,
        accessToken: settings.value.gitSync.accessToken,
      })
      settings.value.gitSync.lastSyncTime = new Date().toISOString()
      await saveSettings()
      console.log('Git同步初始化成功:', result)
    }
    catch (err) {
      error.value = err instanceof Error ? err.message : 'Git同步初始化失败'
      console.error('Failed to initialize Git sync:', err)
      throw err
    }
    finally {
      loading.value = false
    }
  }

  const syncWithGit = async () => {
    try {
      loading.value = true
      const result = await invoke('sync_todos_with_git', {
        settings: settings.value,
      })
      settings.value.gitSync.lastSyncTime = new Date().toISOString()
      await saveSettings()

      // 重新加载数据以获取最新状态
      await loadTodos()

      console.log('Git同步成功:', result)
    }
    catch (err) {
      error.value = err instanceof Error ? err.message : 'Git同步失败'
      console.error('Failed to sync with Git:', err)
      throw err
    }
    finally {
      loading.value = false
    }
  }

  const getGitSyncStatus = async () => {
    try {
      const status = await invoke('get_sync_status')
      return status
    }
    catch (err) {
      console.error('Failed to get Git sync status:', err)
      return { initialized: false, message: 'Failed to get status' }
    }
  }

  const testGitPushAuth = async () => {
    try {
      const result = await invoke('test_git_push_auth', {
        settings: settings.value,
      })
      return result
    }
    catch (err) {
      console.error('Failed to test Git push auth:', err)
      throw err
    }
  }

  const checkLocalSyncFiles = async () => {
    try {
      const result = await invoke('check_local_sync_files')
      return result
    }
    catch (err) {
      console.error('Failed to check local sync files:', err)
      throw err
    }
  }

  const selectSSHKeyFile = async () => {
    try {
      const result = await invoke('select_ssh_key_file')
      return result
    }
    catch (err) {
      console.error('Failed to select SSH key file:', err)
      throw err
    }
  }

  const checkGitRemoteUrl = async () => {
    try {
      const result = await invoke('check_git_remote_url')
      return result
    }
    catch (err) {
      console.error('Failed to check Git remote URL:', err)
      throw err
    }
  }

  const updateGitRemoteUrl = async () => {
    try {
      const result = await invoke('update_git_remote_url', {
        newUrl: settings.value.gitSync.repositoryUrl,
      })
      return result
    }
    catch (err) {
      console.error('Failed to update Git remote URL:', err)
      throw err
    }
  }

  // 自动同步检查
  const checkAutoSync = async () => {
    if (!settings.value.gitSync.enabled || !settings.value.gitSync.autoSync) {
      return
    }

    const lastSync = settings.value.gitSync.lastSyncTime
    if (!lastSync) {
      return
    }

    const lastSyncDate = new Date(lastSync)
    const today = new Date()
    const daysDiff = Math.floor((today.getTime() - lastSyncDate.getTime()) / (1000 * 60 * 60 * 24))

    // 如果超过1天没有同步，自动同步
    if (daysDiff >= 1) {
      try {
        await syncWithGit()
        console.log('自动同步完成')
      }
      catch (err) {
        console.error('自动同步失败:', err)
      }
    }
  }

  // 监听待办事项变化
  watch(todos, scheduleArchiveCheck, { deep: true })

  // 启动时检查自动同步
  checkAutoSync()

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
    exportTodos,
    importTodos,
    setDataSource,
    initializeGitSync,
    syncWithGit,
    getGitSyncStatus,
    testGitPushAuth,
    checkLocalSyncFiles,
    selectSSHKeyFile,
    checkGitRemoteUrl,
    updateGitRemoteUrl,
    checkAutoSync,
  }
})
