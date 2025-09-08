// 待办事项相关的类型定义

export interface TodoItem {
  id: string
  text: string
  completed: boolean
  createdAt: string // ISO 时间字符串
  completedAt?: string // 完成时间
  deadline?: string // 截止时间 ISO 字符串
  parentId?: string // 父项ID，用于树形结构
  children?: TodoItem[] // 子项列表
}

export interface TodoSettings {
  // 颜色配置
  colors: {
    normal: string // 正常状态文字颜色
    warning: string // 24小时内截止的颜色（黄色）
    urgent: string // 1小时内截止的颜色（红色）
    completed: string // 已完成的颜色（浅灰色）
    background: string // 背景颜色
    border: string // 边框颜色
    hover: string // 悬停颜色
  }
  // 回收时间设置（天数）
  archiveDays: number
}

export interface TodoData {
  todos: TodoItem[]
  settings: TodoSettings
}

export interface ArchivedTodoData {
  todos: TodoItem[]
  archivedAt: string
}

// 默认设置
export const defaultTodoSettings: TodoSettings = {
  colors: {
    normal: '#1f2937',
    warning: '#f59e0b', // 黄色
    urgent: '#ef4444', // 红色
    completed: '#9ca3af', // 浅灰色
    background: '#ffffff',
    border: '#e5e7eb',
    hover: '#f3f4f6',
  },
  archiveDays: 30, // 默认30天后归档
}

// 时间工具函数
export const timeUtils = {
  // 检查是否接近截止时间
  getTimeStatus: (deadline: string): 'normal' | 'warning' | 'urgent' => {
    const now = new Date()
    const deadlineDate = new Date(deadline)
    const diffHours = (deadlineDate.getTime() - now.getTime()) / (1000 * 60 * 60)

    if (diffHours <= 1)
      return 'urgent'
    if (diffHours <= 24)
      return 'warning'
    return 'normal'
  },

  // 检查是否需要归档
  shouldArchive: (completedAt: string, archiveDays: number): boolean => {
    const completed = new Date(completedAt)
    const now = new Date()
    const diffDays = (now.getTime() - completed.getTime()) / (1000 * 60 * 60 * 24)
    return diffDays >= archiveDays
  },

  // 格式化时间显示
  formatTime: (isoString: string): string => {
    const date = new Date(isoString)
    return date.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  },
}
